use std::sync::Arc;

use axum::{
  Router,
  body::Body,
  extract::{Path, Query, State},
  middleware::from_fn_with_state,
  response::Response,
  routing::get,
};
use axum_extra::headers::Header;
use headers_accept::Accept;
use http::{HeaderMap, Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
  app::AppContextTrait,
  auth::AuthError,
  errors::RecorderResult,
  web::{
    controller::Controller,
    middleware::native_resources::{NativeResourceOwner, NativeResourceRoute, native_resource_auth},
  },
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
  owner: NativeResourceOwner,
  Query(query): Query<StaticQuery>,
  method: Method,
  headers: HeaderMap,
) -> RecorderResult<Response> {
  if owner.0 != subscriber_id {
    Err(AuthError::PermissionError)?;
  }
  if std::path::Path::new(&path)
    .components()
    .any(|part| !matches!(part, std::path::Component::Normal(_)))
  {
    return Err(AuthError::PermissionError.into());
  }
  let storage = ctx.storage();

  let storage_path = storage.build_subscriber_path(subscriber_id, &path);

  let mut response = deliver(ctx.as_ref(), &method, storage_path.as_str(), &headers, &query, true).await?;
  // Prevent transport transformations from changing the private
  // representation described by range and length headers.
  response
    .headers_mut()
    .insert(http::header::CACHE_CONTROL, http::HeaderValue::from_static("private, no-cache, no-transform"));
  response
    .headers_mut()
    .insert("cross-origin-resource-policy", http::HeaderValue::from_static("cross-origin"));
  Ok(response)
}

async fn serve_public_static(
  State(ctx): State<Arc<dyn AppContextTrait>>,
  Path(path): Path<String>,
  Query(query): Query<StaticQuery>,
  method: Method,
  headers: HeaderMap,
) -> RecorderResult<Response> {
  if std::path::Path::new(&path)
    .components()
    .any(|part| !matches!(part, std::path::Component::Normal(_)))
  {
    return Err(AuthError::PermissionError.into());
  }
  let storage = ctx.storage();

  let storage_path = storage.build_public_path(&path);

  deliver(ctx.as_ref(), &method, storage_path.as_str(), &headers, &query, false).await
}

async fn deliver(ctx: &dyn AppContextTrait, method: &Method, path: &str, headers: &HeaderMap, query: &StaticQuery, private: bool) -> RecorderResult<Response> {
  let optimized = query.optimize.is_some();
  let accept = if headers.contains_key(http::header::ACCEPT) {
    let mut values = headers.get_all(http::header::ACCEPT).iter();
    match Accept::decode(&mut values) {
      Ok(value) if crate::media::negotiation::validate_accept(&value).is_ok() => Some(value),
      _ => {
        let mut response = Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from("Invalid Accept header"))?;
        if optimized {
          crate::storage::merge_vary(response.headers_mut(), "Accept");
        }
        return Ok(response);
      }
    }
  } else {
    None
  };
  if optimized {
    ctx.storage().serve_image(method, path, headers, accept.as_ref(), ctx.media(), private).await
  } else {
    let mut response = ctx.storage().serve_representation(method, path, headers, None).await?;
    response.headers_mut().insert(
      http::header::CACHE_CONTROL,
      http::HeaderValue::from_static(if private { "private, no-cache" } else { "public, no-cache" }),
    );
    Ok(response)
  }
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
  let router = Router::<Arc<dyn AppContextTrait>>::new()
    .route(
      "/subscribers/{subscriber_id}/{*path}",
      get(serve_subscriber_static).layer(from_fn_with_state((ctx, NativeResourceRoute::Image), native_resource_auth)),
    )
    .route("/public/{*path}", get(serve_public_static));

  Ok(Controller::from_nest_router(CONTROLLER_PREFIX, router))
}
