use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use snafu::ResultExt;

use crate::{
    app::AppContextTrait,
    auth::{
        AuthError, AuthService, AuthServiceTrait,
        errors::OidcRequestRedirectUriSnafu,
        oidc::{OidcAuthCallbackPayload, OidcAuthCallbackQuery, OidcAuthRequest},
    },
    errors::RecorderResult,
    extract::http::ForwardedRelatedInfo,
    models::auth::AuthType,
    web::controller::core::Controller,
};

pub const CONTROLLER_PREFIX: &str = "/api/oidc";

async fn oidc_callback(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    Query(query): Query<OidcAuthCallbackQuery>,
) -> Result<Json<OidcAuthCallbackPayload>, AuthError> {
    let auth_service = ctx.auth();
    if let AuthService::Oidc(oidc_auth_service) = auth_service {
        let response = oidc_auth_service
            .extract_authorization_request_callback(query)
            .await?;
        Ok(Json(response))
    } else {
        Err(AuthError::NotSupportAuthMethod {
            supported: vec![auth_service.auth_type()],
            current: AuthType::Oidc,
        })
    }
}

async fn oidc_auth(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    forwarded_info: ForwardedRelatedInfo,
) -> Result<Json<OidcAuthRequest>, AuthError> {
    let auth_service = ctx.auth();
    if let AuthService::Oidc(oidc_auth_service) = auth_service {
        let mut redirect_uri = forwarded_info
            .resolved_origin()
            .ok_or(url::ParseError::EmptyHost)
            .context(OidcRequestRedirectUriSnafu)?;

        redirect_uri.set_path(&format!("{CONTROLLER_PREFIX}/callback"));

        let auth_request = oidc_auth_service
            .build_authorization_request(redirect_uri.as_str())
            .await?;

        {
            oidc_auth_service
                .store_authorization_request(auth_request.clone())
                .await?;
        }

        Ok(Json(auth_request))
    } else {
        Err(AuthError::NotSupportAuthMethod {
            supported: vec![auth_service.auth_type()],
            current: AuthType::Oidc,
        })
    }
}

pub async fn create(_context: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
    let router = Router::<Arc<dyn AppContextTrait>>::new()
        .route("/auth", get(oidc_auth))
        .route("/callback", get(oidc_callback));

    Ok(Controller::from_prefix(CONTROLLER_PREFIX, router))
}
