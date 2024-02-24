use sea_orm::entity::prelude::*;

pub use super::_entities::bangumi::{self, ActiveModel, Entity, Model};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
