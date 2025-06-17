use std::sync::Arc;

use axum::{
    Extension, Router,
    extract::{Path, State},
    middleware::from_fn_with_state,
    response::Response,
    routing::get,
};
use axum_extra::{TypedHeader, headers::Range};

use crate::{
    app::AppContextTrait,
    auth::{AuthError, AuthUserInfo, auth_middleware},
    errors::RecorderResult,
    web::controller::Controller,
};

pub const CONTROLLER_PREFIX: &str = "/api/static";

async fn serve_subscriber_static(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    Path((subscriber_id, path)): Path<(i32, String)>,
    Extension(auth_user_info): Extension<AuthUserInfo>,
    range: Option<TypedHeader<Range>>,
) -> RecorderResult<Response> {
    if subscriber_id != auth_user_info.subscriber_auth.id {
        Err(AuthError::PermissionError)?;
    }

    let storage = ctx.storage();

    let storage_path = storage.build_subscriber_path(subscriber_id, &path);

    storage.serve_file(storage_path, range).await
}

async fn serve_public_static(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    Path(path): Path<String>,
    range: Option<TypedHeader<Range>>,
) -> RecorderResult<Response> {
    let storage = ctx.storage();

    let storage_path = storage.build_public_path(&path);

    storage.serve_file(storage_path, range).await
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
    let router = Router::<Arc<dyn AppContextTrait>>::new()
        .route(
            "/subscribers/{subscriber_id}/{*path}",
            get(serve_subscriber_static).layer(from_fn_with_state(ctx, auth_middleware)),
        )
        .route("/public/{*path}", get(serve_public_static));

    Ok(Controller::from_prefix(CONTROLLER_PREFIX, router))
}
