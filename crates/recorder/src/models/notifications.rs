use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    season: i32,
    episode_size: u32,
    poster_url: Option<String>,
}
