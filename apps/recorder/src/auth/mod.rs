pub mod config;
pub mod errors;
pub mod identity;
pub mod middleware;
pub mod native_resources;
pub mod oidc;
pub mod projection;
pub mod resource;
pub mod service;
pub mod webui;

pub use config::{AuthConfig, OidcAuthConfig, ProviderAuthConfig};
pub use errors::AuthError;
pub use identity::connect_identity_db;
pub use middleware::auth_middleware;
pub use projection::{PublicAuthConfig, PublicAuthProvider};
pub use resource::{ResourceTokenPrincipal, ResourceVerifier};
pub use service::{AuthService, AuthUserInfo};
pub use webui::{CALLBACK_PATH, RUNTIME_CONFIG_PATH, WebuiOrigin, WebuiPolicy};
