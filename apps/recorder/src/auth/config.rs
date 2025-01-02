use jwt_authorizer::OneOrArray;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    #[serde(rename = "basic_user")]
    pub user: String,
    #[serde(rename = "basic_password")]
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OidcAuthConfig {
    #[serde(rename = "oidc_api_issuer")]
    pub issuer: String,
    #[serde(rename = "oidc_api_audience")]
    pub audience: String,
    #[serde(rename = "oidc_extra_scopes")]
    pub extra_scopes: Option<OneOrArray<String>>,
    #[serde(rename = "oidc_extra_claim_key")]
    pub extra_claim_key: Option<String>,
    #[serde(rename = "oidc_extra_claim_value")]
    pub extra_claim_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "auth_type", rename_all = "snake_case")]
pub enum AppAuthConfig {
    Basic(BasicAuthConfig),
    Oidc(OidcAuthConfig),
}
