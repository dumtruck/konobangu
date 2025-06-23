use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use http::StatusCode;

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    extract::http::ForwardedRelatedInfo,
    models::feeds,
    web::controller::Controller,
};

pub const CONTROLLER_PREFIX: &str = "/api/feeds";

async fn rss_handler(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    Path(token): Path<String>,
    forwarded_info: ForwardedRelatedInfo,
) -> RecorderResult<impl IntoResponse> {
    let api_base = forwarded_info
        .resolved_origin()
        .ok_or(RecorderError::MissingOriginError)?;
    let channel = feeds::Model::find_rss_feed_by_token(ctx.as_ref(), &token, &api_base).await?;

    Ok((
        StatusCode::OK,
        [("Content-Type", "application/xml; charset=utf-8")],
        channel.to_string(),
    ))
}

pub async fn create(_ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
    let router = Router::<Arc<dyn AppContextTrait>>::new().route("/rss/{token}", get(rss_handler));

    Ok(Controller::from_prefix(CONTROLLER_PREFIX, router))
}
