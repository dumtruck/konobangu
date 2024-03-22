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

impl worker::AppWorker<SubscriptionWorkerArgs> for SubscriptionWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }
}

#[async_trait]
impl worker::Worker<SubscriptionWorkerArgs> for SubscriptionWorker {
    async fn perform(&self, args: SubscriptionWorkerArgs) -> worker::Result<()> {
        println!("================================================");

        let db = &self.ctx.db;
        let storage = &self.ctx.storage;

        println!("================================================");
        Ok(())
    }
}
