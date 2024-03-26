use std::{ops::Deref, sync::Arc};

use tokio::sync::OnceCell;

use crate::downloaders::defs::ApiClient;

pub struct MikanClient {
    api_client: ApiClient,
}

static MIKAN_CLIENT: OnceCell<Arc<MikanClient>> = OnceCell::const_new();

impl MikanClient {
    pub async fn new(_subscriber_id: i32) -> eyre::Result<Arc<Self>> {
        let res = MIKAN_CLIENT
            .get_or_try_init(|| async {
                ApiClient::new(std::time::Duration::from_millis(50), None)
                    .map(|api_client| Arc::new(Self { api_client }))
            })
            .await?;
        Ok(res.clone())
    }
}

impl Deref for MikanClient {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        &self.api_client
    }
}
