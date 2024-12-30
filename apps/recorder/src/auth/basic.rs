use axum::{http::request::Parts, RequestPartsExt};
use axum_auth::AuthBasic;

use super::{
    config::BasicAuthConfig,
    errors::AuthError,
    service::{AuthService, AuthUserInfo},
};
use crate::models::{auth::AuthType, subscribers::SEED_SUBSCRIBER};

#[derive(Debug)]
pub struct BasicAuthService {
    pub config: BasicAuthConfig,
}

#[async_trait::async_trait]
impl AuthService for BasicAuthService {
    async fn extract_user_info(&self, request: &mut Parts) -> Result<AuthUserInfo, AuthError> {
        if let Ok(AuthBasic((found_user, found_password))) = request.extract().await {
            if self.config.user == found_user
                && self.config.password == found_password.unwrap_or_default()
            {
                return Ok(AuthUserInfo {
                    user_pid: SEED_SUBSCRIBER.to_string(),
                    auth_type: AuthType::Basic,
                });
            }
        }
        Err(AuthError::BasicInvalidCredentials)
    }
}
