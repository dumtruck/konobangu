use std::sync::Arc;

use axum::{extract::Query, http::request::Parts};
use loco_rs::prelude::*;

use crate::{
    app::AppContextExt,
    auth::{
        oidc::{OidcAuthCallbackPayload, OidcAuthCallbackQuery, OidcAuthRequest},
        AppAuthService, AuthError, AuthService,
    },
    extract::http::ForwardedRelatedInfo,
    models::auth::AuthType,
};

async fn oidc_callback(
    State(ctx): State<Arc<AppContext>>,
    Query(query): Query<OidcAuthCallbackQuery>,
) -> Result<Json<OidcAuthCallbackPayload>, AuthError> {
    let auth_service = ctx.get_auth_service();
    if let AppAuthService::Oidc(oidc_auth_service) = auth_service {
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
    State(ctx): State<Arc<AppContext>>,
    parts: Parts,
) -> Result<Json<OidcAuthRequest>, AuthError> {
    let auth_service = ctx.get_auth_service();
    if let AppAuthService::Oidc(oidc_auth_service) = auth_service {
        let mut redirect_uri = ForwardedRelatedInfo::from_request_parts(&parts)
            .resolved_origin()
            .ok_or_else(|| AuthError::OidcRequestRedirectUriError(url::ParseError::EmptyHost))?;

        redirect_uri.set_path("/api/oidc/callback");

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

pub fn routes(state: Arc<AppContext>) -> Routes {
    Routes::new()
        .prefix("/oidc")
        .add("/auth", get(oidc_auth).with_state(state.clone()))
        .add("/callback", get(oidc_callback).with_state(state))
}
