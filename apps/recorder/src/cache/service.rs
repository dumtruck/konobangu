use super::CacheConfig;
use crate::errors::app_error::RResult;

pub struct CacheService {}

impl CacheService {
    pub async fn from_config(_config: CacheConfig) -> RResult<Self> {
        Ok(Self {})
    }
}
