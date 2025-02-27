use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphQLConfig {
    pub depth_limit: Option<usize>,
    pub complexity_limit: Option<usize>,
}
