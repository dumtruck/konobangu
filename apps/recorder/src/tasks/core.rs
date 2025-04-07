use std::{borrow::Cow, sync::Arc};

use async_stream::stream;
use futures::{Stream, StreamExt, pin_mut};
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::{RwLock, mpsc};

use crate::{
    app::AppContextTrait,
    errors::app_error::{RecorderError, RecorderResult},
    models,
};

pub struct TaskMeta {
    pub subscriber_id: i32,
    pub task_id: i32,
    pub task_kind: Cow<'static, str>,
}

pub struct ReplayChannel<T: Send + Sync + Clone + 'static> {
    sender: mpsc::UnboundedSender<T>,
    channels: Arc<RwLock<Vec<mpsc::UnboundedSender<T>>>>,
    buffer: Arc<RwLock<Vec<T>>>,
}

impl<T: Send + Sync + Clone + 'static> ReplayChannel<T> {
    pub fn new(history: Vec<T>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<T>();
        let channels = Arc::new(RwLock::new(Vec::<mpsc::UnboundedSender<T>>::new()));
        let buffer = Arc::new(RwLock::new(history));
        {
            let channels = channels.clone();
            let buffer = buffer.clone();
            tokio::spawn(async move {
                loop {
                    match rx.recv().await {
                        Some(value) => {
                            let mut w = buffer.write().await;
                            let senders = channels.read().await;
                            for s in senders.iter() {
                                if !s.is_closed() {
                                    if let Err(err) = s.send(value.clone()) {
                                        tracing::error!(err = %err, "replay-channel broadcast to other subscribers error");
                                    }
                                }
                            }
                            w.push(value);
                        }
                        None => {
                            drop(rx);
                            let mut cs = channels.write().await;
                            cs.clear();
                            break;
                        }
                    }
                }
            });
        }

        Self {
            sender: tx,
            channels,
            buffer,
        }
    }

    pub fn sender(&self) -> &mpsc::UnboundedSender<T> {
        &self.sender
    }

    pub async fn receiver(&self) -> mpsc::UnboundedReceiver<T> {
        let (tx, rx) = mpsc::unbounded_channel();
        let items = self.buffer.read().await;
        for item in items.iter() {
            if let Err(err) = tx.send(item.clone()) {
                tracing::error!(err = %err, "replay-channel send replay value to other subscribers error");
            }
        }
        if !self.sender.is_closed() {
            let mut sw = self.channels.write().await;
            sw.push(tx);
        }
        rx
    }

    pub async fn close(&self) {
        let mut senders = self.channels.write().await;
        senders.clear();
    }
}

pub trait StreamTaskCoreTrait: Sized {
    type Request: Serialize + DeserializeOwned;
    type Item: Serialize + DeserializeOwned;

    fn task_id(&self) -> i32;

    fn task_kind(&self) -> &str;

    fn new(meta: TaskMeta, request: Self::Request) -> Self;

    fn request(&self) -> &Self::Request;
}

pub trait StreamTaskReplayLayoutTrait: StreamTaskCoreTrait {
    fn history(&self) -> &[Arc<RecorderResult<Self::Item>>];

    fn resume_from_model(
        task: models::tasks::Model,
        stream_items: Vec<models::task_stream_item::Model>,
    ) -> RecorderResult<Self>;

    fn running_receiver(
        &self,
    ) -> impl Future<Output = Option<mpsc::UnboundedReceiver<Arc<RecorderResult<Self::Item>>>>>;

    #[allow(clippy::type_complexity)]
    fn init_receiver(
        &self,
    ) -> impl Future<
        Output = (
            mpsc::UnboundedSender<Arc<RecorderResult<Self::Item>>>,
            mpsc::UnboundedReceiver<Arc<RecorderResult<Self::Item>>>,
        ),
    >;

    fn serialize_request(request: Self::Request) -> RecorderResult<serde_json::Value> {
        serde_json::to_value(request).map_err(RecorderError::from)
    }

    fn serialize_item(item: RecorderResult<Self::Item>) -> RecorderResult<serde_json::Value> {
        serde_json::to_value(item).map_err(RecorderError::from)
    }

    fn deserialize_request(request: serde_json::Value) -> RecorderResult<Self::Request> {
        serde_json::from_value(request).map_err(RecorderError::from)
    }

    fn deserialize_item(item: serde_json::Value) -> RecorderResult<RecorderResult<Self::Item>> {
        serde_json::from_value(item).map_err(RecorderError::from)
    }
}

pub trait StreamTaskRunnerTrait: StreamTaskCoreTrait {
    fn run(
        context: Arc<dyn AppContextTrait>,
        request: &Self::Request,
        history: &[Arc<RecorderResult<Self::Item>>],
    ) -> impl Stream<Item = RecorderResult<Self::Item>>;
}

pub trait StreamTaskReplayRunnerTrait: StreamTaskRunnerTrait + StreamTaskReplayLayoutTrait {
    fn run_shared(
        &self,
        context: Arc<dyn AppContextTrait>,
    ) -> impl Stream<Item = Arc<RecorderResult<Self::Item>>> {
        stream! {
            if let Some(mut receiver) = self.running_receiver().await {
                while let Some(item) = receiver.recv().await {
                    yield item
                }
            } else {
                let (tx, _) = self.init_receiver().await;
                let stream = Self::run(context, self.request(), self.history());

                pin_mut!(stream);

                while let Some(item) = stream.next().await {
                    let item = Arc::new(item);
                    if let Err(err) = tx.send(item.clone()) {
                        tracing::error!(task_id = self.task_id(), task_kind = self.task_kind(), err = %err, "run shared send error");
                    }
                    yield item
                }
            };

        }
    }
}

pub struct StandardStreamTaskReplayLayout<Request, Item>
where
    Request: Serialize + DeserializeOwned,
    Item: Serialize + DeserializeOwned + Sync + Send + 'static,
{
    pub meta: TaskMeta,
    pub request: Request,
    pub history: Vec<Arc<RecorderResult<Item>>>,
    #[allow(clippy::type_complexity)]
    pub channel: Arc<RwLock<Option<ReplayChannel<Arc<RecorderResult<Item>>>>>>,
}

impl<Request, Item> StreamTaskCoreTrait for StandardStreamTaskReplayLayout<Request, Item>
where
    Request: Serialize + DeserializeOwned,
    Item: Serialize + DeserializeOwned + Sync + Send + 'static,
{
    type Request = Request;
    type Item = Item;

    fn task_id(&self) -> i32 {
        self.meta.task_id
    }

    fn request(&self) -> &Self::Request {
        &self.request
    }

    fn task_kind(&self) -> &str {
        &self.meta.task_kind
    }

    fn new(meta: TaskMeta, request: Self::Request) -> Self {
        Self {
            meta,
            request,
            history: vec![],
            channel: Arc::new(RwLock::new(None)),
        }
    }
}

impl<Request, Item> StreamTaskReplayLayoutTrait for StandardStreamTaskReplayLayout<Request, Item>
where
    Request: Serialize + DeserializeOwned,
    Item: Serialize + DeserializeOwned + Sync + Send + 'static,
{
    fn history(&self) -> &[Arc<RecorderResult<Self::Item>>] {
        &self.history
    }

    fn resume_from_model(
        task: models::tasks::Model,
        stream_items: Vec<models::task_stream_item::Model>,
    ) -> RecorderResult<Self> {
        Ok(Self {
            meta: TaskMeta {
                task_id: task.id,
                subscriber_id: task.subscriber_id,
                task_kind: Cow::Owned(task.task_type),
            },
            request: Self::deserialize_request(task.request_data)?,
            history: stream_items
                .into_iter()
                .map(|m| Self::deserialize_item(m.item).map(Arc::new))
                .collect::<RecorderResult<Vec<_>>>()?,
            channel: Arc::new(RwLock::new(None)),
        })
    }

    async fn running_receiver(
        &self,
    ) -> Option<mpsc::UnboundedReceiver<Arc<RecorderResult<Self::Item>>>> {
        if let Some(channel) = self.channel.read().await.as_ref() {
            Some(channel.receiver().await)
        } else {
            None
        }
    }

    async fn init_receiver(
        &self,
    ) -> (
        mpsc::UnboundedSender<Arc<RecorderResult<Self::Item>>>,
        mpsc::UnboundedReceiver<Arc<RecorderResult<Self::Item>>>,
    ) {
        let channel = ReplayChannel::new(self.history.clone());
        let rx = channel.receiver().await;
        let sender = channel.sender().clone();

        {
            {
                let mut w = self.channel.write().await;
                *w = Some(channel);
            }
        }
        (sender, rx)
    }
}
