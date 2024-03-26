use regex::Regex;
use sea_orm::entity::prelude::*;

pub use super::entities::bangumi::*;
use crate::models::downloads;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl BangumiFilter {
    pub fn is_match(&self, title: &str) -> eyre::Result<bool> {
        if let Some(regex_filters) = &self.regex_filters {
            let combined_regex = Regex::new(&regex_filters.join("|"))?;
            if combined_regex.is_match(title) {
                return Ok(true);
            }
        } else if let Some(plain_filters) = &self.plaintext_filters {
            for f in plain_filters {
                if title.contains(f) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl Model {
    pub async fn search_all() {}
    pub async fn match_list(dnlds: Vec<downloads::Model>) {}
}
