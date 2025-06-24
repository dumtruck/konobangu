use std::sync::Arc;

use axum::{
    Extension, Router,
    extract::{Path, Query, State},
    middleware::from_fn_with_state,
    response::Response,
    routing::get,
};
use axum_extra::{TypedHeader, headers::Range};
use headers_accept::Accept;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    auth::{AuthError, AuthUserInfo, auth_middleware},
    errors::RecorderResult,
    web::controller::Controller,
};

pub const CONTROLLER_PREFIX: &str = "/api/static";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum OptimizeType {
    #[serde(rename = "accept")]
    AcceptHeader,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StaticQuery {
    optimize: Option<OptimizeType>,
}

async fn serve_subscriber_static(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    Path((subscriber_id, path)): Path<(i32, String)>,
    Extension(auth_user_info): Extension<AuthUserInfo>,
    Query(query): Query<StaticQuery>,
    range: Option<TypedHeader<Range>>,
    accept: Option<TypedHeader<Accept>>,
) -> RecorderResult<Response> {
    if subscriber_id != auth_user_info.subscriber_auth.id {
        Err(AuthError::PermissionError)?;
    }
    let storage = ctx.storage();
    let media = ctx.media();

    let storage_path = storage.build_subscriber_path(subscriber_id, &path);

    if query
        .optimize
        .is_some_and(|optimize| optimize == OptimizeType::AcceptHeader)
        && storage_path
            .extension()
            .is_some_and(|ext| media.is_legacy_image_format(ext))
        && let Some(TypedHeader(accept)) = accept
    {
        storage
            .serve_optimized_image(storage_path, range, accept)
            .await
    } else {
        storage.serve_file(storage_path, range).await
    }
}

async fn serve_public_static(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    Path(path): Path<String>,
    Query(query): Query<StaticQuery>,
    range: Option<TypedHeader<Range>>,
    accept: Option<TypedHeader<Accept>>,
) -> RecorderResult<Response> {
    let storage = ctx.storage();
    let media = ctx.media();

    let storage_path = storage.build_public_path(&path);

    if query
        .optimize
        .is_some_and(|optimize| optimize == OptimizeType::AcceptHeader)
        && storage_path
            .extension()
            .is_some_and(|ext| media.is_legacy_image_format(ext))
        && let Some(TypedHeader(accept)) = accept
    {
        storage
            .serve_optimized_image(storage_path, range, accept)
            .await
    } else {
        storage.serve_file(storage_path, range).await
    }
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
    let router = Router::<Arc<dyn AppContextTrait>>::new()
        .route(
            "/subscribers/{subscriber_id}/{*path}",
            get(serve_subscriber_static).layer(from_fn_with_state(ctx, auth_middleware)),
        )
        .route("/public/{*path}", get(serve_public_static));

    Ok(Controller::from_nest_router(CONTROLLER_PREFIX, router))
}
