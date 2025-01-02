use loco_rs::prelude::*;

use crate::{models::subscribers, views::subscribers::CurrentResponse};

async fn current(State(ctx): State<AppContext>) -> Result<impl IntoResponse> {
    let subscriber = subscribers::Model::find_root(&ctx).await?;
    format::json(CurrentResponse::new(&subscriber))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("subscribers")
        .add("/current", get(current))
}
