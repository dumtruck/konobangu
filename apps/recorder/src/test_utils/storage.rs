use opendal::{Operator, layers::LoggingLayer};

use crate::{errors::RecorderResult, storage::StorageServiceTrait};

pub struct TestingStorageService {
    operator: Operator,
}

impl TestingStorageService {
    pub fn new() -> RecorderResult<Self> {
        let op = Operator::new(opendal::services::Memory::default())?
            .layer(LoggingLayer::default())
            .finish();

        Ok(Self { operator: op })
    }
}

#[async_trait::async_trait]
impl StorageServiceTrait for TestingStorageService {
    fn get_operator(&self) -> RecorderResult<Operator> {
        Ok(self.operator.clone())
    }
}

pub async fn build_testing_storage_service() -> RecorderResult<TestingStorageService> {
    TestingStorageService::new()
}
