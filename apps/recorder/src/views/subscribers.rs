use serde::{Deserialize, Serialize};

use crate::models::subscribers;

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrentResponse {}

impl CurrentResponse {
    #[must_use]
    pub fn new(_user: &subscribers::Model) -> Self {
        Self {}
    }
}
