use futures::{TryStreamExt, pin_mut};
use loco_rs::prelude::*;

use crate::{
    extract::mikan::{
        MikanAuthSecrecy
    },
};

#[derive(Debug)]
pub struct CreateMikanRSSFromMyBangumiTask {
    subscriber_id: i32,
    task_id: String,
    auth_secrecy: MikanAuthSecrecy,
}

#[async_trait::async_trait]
impl Task for CreateMikanRSSFromMyBangumiTask {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: format!(
                "create-mikan-rss-from-my-bangumi-{}-{}",
                self.subscriber_id, self.task_id
            ),
            detail: "create mikan rss from my bangumi page for {} {}".to_string(),
        }
    }

    async fn run(&self, app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        let mikan_client = app_context
            .get_mikan_client()
            .fork_with_auth(self.auth_secrecy.clone())?;

        {
            let bangumi_metas = extract_mikan_bangumis_meta_from_my_bangumi_page(
                &mikan_client,
                mikan_client
                    .base_url()
                    .join("/Home/MyBangumi")
                    .map_err(loco_rs::Error::wrap)?,
            );

            pin_mut!(bangumi_metas);

            let bangumi_metas = bangumi_metas.try_collect::<Vec<_>>().await?;

            tokio::sync::broadcast::
        }

        Ok(())
    }
}
