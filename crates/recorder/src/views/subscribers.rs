use serde::{Deserialize, Serialize};

use crate::models::entities::subscribers;

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrentResponse {
    pub pid: String,
    pub display_name: String,
}

impl CurrentResponse {
    #[must_use]
    pub fn new(user: &subscribers::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            display_name: user.display_name.to_string(),
        }
    }
}
