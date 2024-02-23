use sea_orm::entity::prelude::*;

pub use super::_entities::episodes::{self, ActiveModel, Entity, Model};

#[async_trait::async_trait]
impl ActiveModelBehavior for super::_entities::episodes::ActiveModel {}
