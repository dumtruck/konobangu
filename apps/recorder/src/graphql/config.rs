use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppGraphQLConfig {
    pub depth_limit: Option<usize>,
    pub complexity_limit: Option<usize>,
}
