use loco_rs::app::AppContext;
use sea_orm::{entity::prelude::*, ActiveValue, TryIntoModel};

pub use super::entities::bangumi::*;

impl Model {
    pub async fn get_or_insert_from_mikan<F>(
        ctx: &AppContext,
        subscription_id: i32,
        mikan_bangumi_id: String,
        mikan_fansub_id: String,
        f: F,
    ) -> eyre::Result<Model>
    where
        F: AsyncFnOnce(&mut ActiveModel) -> eyre::Result<()>,
    {
        let db = &ctx.db;
        if let Some(existed) = Entity::find()
            .filter(
                Column::MikanBangumiId
                    .eq(Some(mikan_bangumi_id.clone()))
                    .and(Column::MikanFansubId.eq(Some(mikan_fansub_id.clone()))),
            )
            .one(db)
            .await?
        {
            Ok(existed)
        } else {
            let mut bgm = ActiveModel {
                mikan_bangumi_id: ActiveValue::Set(Some(mikan_bangumi_id)),
                mikan_fansub_id: ActiveValue::Set(Some(mikan_fansub_id)),
                subscription_id: ActiveValue::Set(subscription_id),
                ..Default::default()
            };
            f(&mut bgm).await?;
            let bgm: Model = bgm.save(db).await?.try_into_model()?;
            Ok(bgm)
        }
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
