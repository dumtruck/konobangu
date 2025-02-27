use std::borrow::Cow;

use futures::{TryStreamExt, pin_mut};

use super::core::{Task, TaskVars};
use crate::{
    app::AppContext,
    errors::RResult,
    extract::mikan::{
        MikanAuthSecrecy, web_extract::extract_mikan_bangumis_meta_from_my_bangumi_page,
    },
};

#[derive(Debug)]
pub struct CreateMikanRSSFromMyBangumiTask {
    pub subscriber_id: i32,
    pub task_id: String,
    pub auth_secrecy: MikanAuthSecrecy,
}

#[async_trait::async_trait]
impl Task for CreateMikanRSSFromMyBangumiTask {
    fn task_name() -> Cow<'static, str> {
        Cow::Borrowed("create-mikan-rss-from-my-bangumi")
    }

    fn task_id(&self) -> &str {
        &self.task_id
    }

    async fn run(&self, app_context: &AppContext, _vars: &TaskVars) -> RResult<()> {
        let mikan_client = app_context
            .mikan
            .fork_with_auth(self.auth_secrecy.clone())?;

        {
            let bangumi_metas = extract_mikan_bangumis_meta_from_my_bangumi_page(
                &mikan_client,
                mikan_client.base_url().join("/Home/MyBangumi")?,
            );

            pin_mut!(bangumi_metas);

            let _bangumi_metas = bangumi_metas.try_collect::<Vec<_>>().await?;
        }

        Ok(())
    }
}
