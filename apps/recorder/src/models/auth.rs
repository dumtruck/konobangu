use sea_orm::entity::prelude::*;

pub use super::entities::auth::*;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
