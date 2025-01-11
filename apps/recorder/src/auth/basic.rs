use async_trait::async_trait;
use axum::http::{request::Parts, HeaderValue};
use base64::{self, Engine};
use reqwest::header::AUTHORIZATION;

use super::{
    config::BasicAuthConfig,
    errors::AuthError,
    service::{AuthService, AuthUserInfo},
};
use crate::models::{auth::AuthType, subscribers::SEED_SUBSCRIBER};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthBasic {
    pub user: String,
    pub password: Option<String>,
}

impl AuthBasic {
    fn decode_request_parts(req: &mut Parts) -> Result<Self, AuthError> {
        let authorization = req
            .headers
            .get(AUTHORIZATION)
            .and_then(|s| s.to_str().ok())
            .ok_or_else(|| AuthError::BasicInvalidCredentials)?;

        let split = authorization.split_once(' ');

        match split {
            Some((name, contents)) if name == "Basic" => {
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(contents)
                    .map_err(|_| AuthError::BasicInvalidCredentials)?;

                let decoded =
                    String::from_utf8(decoded).map_err(|_| AuthError::BasicInvalidCredentials)?;

                Ok(if let Some((user, password)) = decoded.split_once(':') {
                    Self {
                        user: String::from(user),
                        password: Some(String::from(password)),
                    }
                } else {
                    Self {
                        user: decoded,
                        password: None,
                    }
                })
            }
            _ => Err(AuthError::BasicInvalidCredentials),
        }
    }
}

#[derive(Debug)]
pub struct BasicAuthService {
    pub config: BasicAuthConfig,
}

#[async_trait]
impl AuthService for BasicAuthService {
    async fn extract_user_info(&self, request: &mut Parts) -> Result<AuthUserInfo, AuthError> {
        if let Ok(AuthBasic {
            user: found_user,
            password: found_password,
        }) = AuthBasic::decode_request_parts(request)
        {
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

    fn www_authenticate_header_value(&self) -> Option<HeaderValue> {
        Some(HeaderValue::from_static(r#"Basic realm="konobangu""#))
    }
}
