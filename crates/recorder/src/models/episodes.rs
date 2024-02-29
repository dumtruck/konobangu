use sea_orm::entity::prelude::*;

pub use super::entities::episodes::*;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
