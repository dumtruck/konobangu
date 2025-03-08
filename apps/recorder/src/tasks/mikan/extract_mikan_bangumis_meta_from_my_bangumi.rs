use std::sync::Arc;

use futures::Stream;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    app::AppContextTrait,
    errors::RResult,
    extract::mikan::{MikanAuthSecrecy, MikanBangumiMeta, web_extract},
    tasks::core::{StandardStreamTaskReplayLayout, StreamTaskRunnerTrait},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractMikanBangumisMetaFromMyBangumiRequest {
    pub my_bangumi_page_url: Url,
    pub auth_secrecy: Option<MikanAuthSecrecy>,
}

pub type ExtractMikanBangumisMetaFromMyBangumiTask =
    StandardStreamTaskReplayLayout<ExtractMikanBangumisMetaFromMyBangumiRequest, MikanBangumiMeta>;

impl StreamTaskRunnerTrait for ExtractMikanBangumisMetaFromMyBangumiTask {
    fn run(
        context: Arc<dyn AppContextTrait>,
        request: &Self::Request,
        history: &[Arc<RResult<Self::Item>>],
    ) -> impl Stream<Item = RResult<Self::Item>> {
        let context = context.clone();
        web_extract::extract_mikan_bangumis_meta_from_my_bangumi_page(
            context,
            request.my_bangumi_page_url.clone(),
            request.auth_secrecy.clone(),
            history,
        )
    }
}
