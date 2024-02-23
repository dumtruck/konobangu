use sea_orm::entity::prelude::*;

pub use super::_entities::subscriptions::{self, ActiveModel, Entity, Model};

#[async_trait::async_trait]
impl ActiveModelBehavior for super::_entities::subscriptions::ActiveModel {}
