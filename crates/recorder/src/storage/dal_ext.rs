use std::sync::Arc;

use eyre::Context;
use loco_rs::app::AppContext;
use tokio::sync::OnceCell;

use crate::{
    config::{deserialize_key_path_from_loco_rs_config, AppDalConf},
    storage::DalContext,
};

static APP_DAL_CONTEXT: OnceCell<Arc<DalContext>> = OnceCell::const_new();

#[async_trait::async_trait]
pub trait AppContextDalExt {
    async fn get_dal(&self) -> eyre::Result<Arc<DalContext>>;
    async fn get_dal_unwrap(&self) -> Arc<DalContext>;
    async fn init_dal(&self) -> eyre::Result<Arc<DalContext>> {
        self.get_dal().await.wrap_err("dal context failed to init")
    }
}

#[async_trait::async_trait]
impl AppContextDalExt for AppContext {
    async fn get_dal(&self) -> eyre::Result<Arc<DalContext>> {
        let context = APP_DAL_CONTEXT
            .get_or_try_init(|| async {
                deserialize_key_path_from_loco_rs_config::<AppDalConf>(&["dal"], &self.config)
                    .map(|dal_conf| Arc::new(DalContext::new(dal_conf)))
            })
            .await?;
        Ok(context.clone())
    }

    async fn get_dal_unwrap(&self) -> Arc<DalContext> {
        self.get_dal()
            .await
            .unwrap_or_else(|e| panic!("dal context failed to init: {}", e))
    }
}
