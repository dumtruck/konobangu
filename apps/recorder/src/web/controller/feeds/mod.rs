use std::sync::Arc;

use axum::{
  Router,
  extract::{Path, State},
  middleware::from_fn_with_state,
  response::IntoResponse,
  routing::get,
};
use http::StatusCode;

use crate::{app::AppContextTrait, errors::RecorderResult, models::feeds, web::controller::Controller};

pub const CONTROLLER_PREFIX: &str = "/api/feeds";

pub(crate) fn rss_url(api_base: &url::Url, token: &str) -> Result<url::Url, url::ParseError> {
  // Fixed bases may include a reverse-proxy service prefix. A root-relative
  // join would discard it, while an unescaped token could change the path.
  let mut base = api_base.clone();
  let path = format!("{}/", base.path().trim_end_matches('/'));
  base.set_path(&path);
  let mut target = base.join(&format!("{}/rss/", CONTROLLER_PREFIX.trim_start_matches('/')))?;
  target
    .path_segments_mut()
    .map_err(|_| url::ParseError::RelativeUrlWithCannotBeABaseBase)?
    .pop_if_empty()
    .push(token);
  Ok(target)
}

async fn rss_handler(
  State(ctx): State<Arc<dyn AppContextTrait>>,
  Path(token): Path<String>,
  owner: crate::web::middleware::native_resources::NativeResourceOwner,
  headers: http::HeaderMap,
) -> RecorderResult<impl IntoResponse> {
  // RSS self-links describe this API; they never select a trusted browser
  // source, callback or authentication target.
  let server = &ctx.config().server;
  let api_base = server.external_base_url.resolve_url(&headers, &server.host, server.port)?;
  let channel = feeds::Model::find_rss_feed_by_token(ctx.as_ref(), &token, &api_base, owner.0)
    .await
    .map_err(|error| match error {
      crate::errors::RecorderError::ModelEntityNotFound { .. } => crate::auth::AuthError::PermissionError.into(),
      error => error,
    })?;

  Ok((
    StatusCode::OK,
    [
      ("Content-Type", "application/xml; charset=utf-8"),
      ("Cache-Control", "no-store"),
      ("Referrer-Policy", "no-referrer"),
      ("Vary", "Host, Forwarded, X-Forwarded-Host, X-Forwarded-Proto"),
    ],
    channel.to_string(),
  ))
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
  let router = Router::<Arc<dyn AppContextTrait>>::new().route(
    "/rss/{token}",
    get(rss_handler).layer(from_fn_with_state(
      (ctx, crate::web::middleware::native_resources::NativeResourceRoute::Rss),
      crate::web::middleware::native_resources::native_resource_auth,
    )),
  );

  Ok(Controller::from_nest_router(CONTROLLER_PREFIX, router))
}

#[cfg(test)]
mod tests {
  use securitydept_utils::base_url::ExternalBaseUrl;

  use super::*;

  #[test]
  fn rss_self_links_use_api_address_without_a_browser_source() {
    let mut headers = http::HeaderMap::new();
    headers.insert("host", "api.local:5001".parse().unwrap());
    headers.insert("x-forwarded-host", "api.region.example".parse().unwrap());
    headers.insert("x-forwarded-proto", "https".parse().unwrap());
    headers.insert("origin", "https://ui.unrelated.example".parse().unwrap());
    let base = ExternalBaseUrl::Auto.resolve_url(&headers, "127.0.0.1", 5001).unwrap();
    assert_eq!(rss_url(&base, "fixture").unwrap().as_str(), "https://api.region.example/api/feeds/rss/fixture");
    headers.remove("origin");
    let fixed = ExternalBaseUrl::Fixed("https://fixed-api.example".into());
    let base = fixed.resolve_url(&headers, "127.0.0.1", 5001).unwrap();
    assert_eq!(rss_url(&base, "fixture").unwrap().as_str(), "https://fixed-api.example/api/feeds/rss/fixture");
  }

  #[test]
  fn rss_self_links_preserve_service_prefixes_and_encode_the_token_as_one_segment() {
    for prefix in ["https://api.example/base", "https://api.example/base/", "https://api.example/base///"] {
      let base = url::Url::parse(prefix).unwrap();
      assert_eq!(rss_url(&base, "fixture").unwrap().as_str(), "https://api.example/base/api/feeds/rss/fixture");
      assert_eq!(
        rss_url(&base, "x/y?z#space %").unwrap().as_str(),
        "https://api.example/base/api/feeds/rss/x%2Fy%3Fz%23space%20%25"
      );
    }
    let base = url::Url::parse("https://api.example/base%20path/nested").unwrap();
    assert_eq!(
      rss_url(&base, "fixture").unwrap().as_str(),
      "https://api.example/base%20path/nested/api/feeds/rss/fixture"
    );
  }
}
