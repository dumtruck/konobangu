use sea_orm::entity::prelude::*;

pub use super::entities::bangumi::*;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
