use std::sync::Arc;

use axum::{
  extract::{Request, State},
  http::header,
  middleware::Next,
  response::{IntoResponse, Response},
};

use crate::app::AppContextTrait;

/// Authenticates a private API request.
///
/// Basic challenges are restricted to the bundled Provider login endpoint.
pub async fn auth_middleware(State(ctx): State<Arc<dyn AppContextTrait>>, request: Request, next: Next) -> Response {
  let auth_service = ctx.auth();

  let (mut parts, body) = request.into_parts();

  let mut response = match auth_service.extract_user_info(&mut parts).await {
    Ok(auth_user_info) => {
      let mut request = Request::from_parts(parts, body);
      request.extensions_mut().insert(auth_user_info);
      next.run(request).await
    }
    Err(auth_error) => auth_error.into_response(),
  };

  let challenge = if response.status() == axum::http::StatusCode::UNAUTHORIZED {
    auth_service.www_authenticate_header_value()
  } else {
    None
  };
  if let Some(header_value) = challenge {
    response.headers_mut().insert(header::WWW_AUTHENTICATE, header_value);
  }

  response
}
