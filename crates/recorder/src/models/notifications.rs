use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    official_title: String,
    season: i32,
    episode_size: u32,
    poster_url: Option<String>,
}
