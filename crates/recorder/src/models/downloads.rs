use sea_orm::ActiveModelBehavior;

use crate::models::_entities::downloads::*;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
