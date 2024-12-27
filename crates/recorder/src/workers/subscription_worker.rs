use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::subscriptions;

pub struct SubscriptionWorker {
    pub ctx: AppContext,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionWorkerArgs {
    pub subscription: subscriptions::Model,
}

#[async_trait]

impl BackgroundWorker<SubscriptionWorkerArgs> for SubscriptionWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, _args: SubscriptionWorkerArgs) -> Result<()> {
        println!("================================================");

        let _db = &self.ctx.db;
        let _storage = &self.ctx.storage;

        println!("================================================");
        Ok(())
    }
}
