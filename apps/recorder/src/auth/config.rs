use jwt_authorizer::OneOrArray;
use serde::{Deserialize, Serialize};
use serde_with::{NoneAsEmptyString, serde_as};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    #[serde(rename = "basic_user")]
    pub user: String,
    #[serde(rename = "basic_password")]
    pub password: String,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OidcAuthConfig {
    #[serde(rename = "oidc_issuer")]
    pub issuer: String,
    #[serde(rename = "oidc_audience")]
    pub audience: String,
    #[serde(rename = "oidc_client_id")]
    pub client_id: String,
    #[serde(rename = "oidc_client_secret")]
    pub client_secret: String,
    #[serde(rename = "oidc_extra_scopes")]
    pub extra_scopes: Option<OneOrArray<String>>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "oidc_extra_claim_key")]
    pub extra_claim_key: Option<String>,
    #[serde(rename = "oidc_extra_claim_value")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub extra_claim_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "auth_type", rename_all = "snake_case")]
pub enum AuthConfig {
    Basic(BasicAuthConfig),
    Oidc(OidcAuthConfig),
}
