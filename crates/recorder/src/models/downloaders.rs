use sea_orm::prelude::*;
use url::Url;

pub use crate::models::entities::downloaders::*;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn endpoint_url(&self) -> Result<Url, url::ParseError> {
        let url = Url::parse(&self.endpoint)?;
        Ok(url)
    }
}
