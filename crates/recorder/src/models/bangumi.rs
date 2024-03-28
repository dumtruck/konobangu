use std::collections::HashSet;

use itertools::Itertools;
use regex::Regex;
use sea_orm::entity::prelude::*;

pub use super::entities::bangumi::*;

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
    pub fn get_unique_key(&self) -> BangumiUniqueKey {
        BangumiUniqueKey {
            official_title: self.official_title.clone(),
            season: self.season,
            fansub: self.fansub.clone(),
        }
    }

    pub async fn find_by_unique_keys(
        db: &DatabaseConnection,
        unique_keys: impl Iterator<Item = &BangumiUniqueKey>,
    ) -> eyre::Result<Vec<Self>> {
        let unique_keys = unique_keys.collect::<HashSet<_>>();
        let mut found = Entity::find()
            .filter(Column::OfficialTitle.is_in(unique_keys.iter().map(|k| &k.official_title)))
            .all(db)
            .await?;

        found = found
            .into_iter()
            .filter(|m| unique_keys.contains(&m.get_unique_key()))
            .collect_vec();

        Ok(found)
    }
}
