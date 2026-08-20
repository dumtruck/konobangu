use std::sync::Arc;

use axum::{
  extract::{FromRequestParts, Path, Request, State},
  middleware::Next,
  response::{IntoResponse, Response},
};

use crate::{
  app::AppContextTrait,
  auth::{AuthError, AuthUserInfo, auth_middleware},
};

#[derive(Clone, Copy)]
pub enum NativeResourceRoute {
  Image,
  Rss,
}

#[derive(Clone, Copy)]
pub struct NativeResourceOwner(pub i32);

impl<S: Send + Sync> FromRequestParts<S> for NativeResourceOwner {
  type Rejection = AuthError;

  async fn from_request_parts(parts: &mut http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
    parts
      .extensions
      .get::<Self>()
      .copied()
      .or_else(|| parts.extensions.get::<AuthUserInfo>().map(|user| Self(user.subscriber_auth.subscriber_id)))
      .ok_or(AuthError::PermissionError)
  }
}

/// Only explicitly enrolled native resource routes accept query credentials.
pub async fn native_resource_auth(State((ctx, route)): State<(Arc<dyn AppContextTrait>, NativeResourceRoute)>, request: Request, next: Next) -> Response {
  let mut tokens = url::form_urlencoded::parse(request.uri().query().unwrap_or_default().as_bytes())
    .filter(|(key, _)| key == "resource_token")
    .map(|(_, value)| value.into_owned());
  let Some(token) = tokens.next() else {
    return auth_middleware(State(ctx), request, next).await;
  };
  if tokens.next().is_some() {
    return AuthError::NativeResourceTokenInvalid.into_response();
  }
  drop(tokens);
  let (mut parts, body) = request.into_parts();
  let resource = match route {
    NativeResourceRoute::Image => Path::<(i32, String)>::from_request_parts(&mut parts, &ctx)
      .await
      .map(|Path((owner, path))| (Some(owner), path)),
    NativeResourceRoute::Rss => Path::<String>::from_request_parts(&mut parts, &ctx)
      .await
      .map(|Path(token)| (None, format!("feeds/rss/{token}"))),
  };
  let Ok((expected_owner, path)) = resource else {
    return AuthError::NativeResourceTokenInvalid.into_response();
  };
  let owner = match ctx.auth().resource_grants.verify_resource(&token, expected_owner, &path, &parts.method) {
    Ok(owner) => owner,
    Err(error) => return error.into_response(),
  };
  if parts.headers.contains_key(http::header::AUTHORIZATION) {
    return AuthError::NativeResourceTokenInvalid.into_response();
  }
  let mut request = Request::from_parts(parts, body);
  request.extensions_mut().insert(NativeResourceOwner(owner));
  next.run(request).await
}
