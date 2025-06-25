use std::sync::Arc;

use sea_orm::{DeriveActiveEnum, DeriveDisplay, EnumIter, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    task::{AsyncTaskTrait, registry::media::OptimizeImageTask},
};

#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Copy,
    DeriveActiveEnum,
    DeriveDisplay,
    EnumIter,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum SystemTaskType {
    #[serde(rename = "optimize_image")]
    #[sea_orm(string_value = "optimize_image")]
    OptimizeImage,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromJsonQueryResult)]
pub enum SystemTask {
    #[serde(rename = "optimize_image")]
    OptimizeImage(OptimizeImageTask),
}

impl SystemTask {
    pub async fn run(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        match self {
            Self::OptimizeImage(task) => task.run(ctx).await,
        }
    }
}
