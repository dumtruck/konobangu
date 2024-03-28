use axum::Router as AxumRouter;
use loco_rs::app::{AppContext, Initializer};

use crate::storage::AppContextDalExt;

pub struct AppDalInitializer;

#[async_trait::async_trait]
impl Initializer for AppDalInitializer {
    fn name(&self) -> String {
        "AppDalInitializer".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> loco_rs::Result<()> {
        ctx.init_dal().await?;
        Ok(())
    }

    async fn after_routes(
        &self,
        router: AxumRouter,
        _ctx: &AppContext,
    ) -> loco_rs::Result<AxumRouter> {
        Ok(router)
    }
}
