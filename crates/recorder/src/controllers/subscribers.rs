use loco_rs::prelude::*;

use crate::{models::_entities::subscribers, views::subscribers::CurrentResponse};

async fn current(State(ctx): State<AppContext>) -> Result<Json<CurrentResponse>> {
    let subscriber = subscribers::Model::find_root(&ctx.db).await?;
    format::json(CurrentResponse::new(&subscriber))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("subscribers")
        .add("/current", get(current))
}
