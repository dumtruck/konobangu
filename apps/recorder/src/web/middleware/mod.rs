pub mod catch_panic;
pub mod compression;
pub mod cors;
pub mod etag;
pub mod format;
pub mod logger;
pub mod remote_ip;
pub mod request_id;
pub mod secure_headers;
pub mod timeout;

use std::sync::Arc;

use axum::Router;
use serde::{Deserialize, Serialize};

use crate::{app::AppContext, errors::RResult};

/// Trait representing the behavior of middleware components in the application.
/// When implementing a new middleware, make sure to go over this checklist:
/// * The name of the middleware should be an ID that is similar to the field
///   name in configuration (look at how `serde` calls it)
/// * Default value implementation should be paired with `serde` default
///   handlers and default serialization implementation. Which means deriving
///   `Default` will _not_ work. You can use `serde_json` and serialize a new
///   config from an empty value, which will cause `serde` default value
///   handlers to kick in.
/// * If you need completely blank values for configuration (for example for
///   testing), implement an `::empty() -> Self` call ad-hoc.
pub trait MiddlewareLayer {
    /// Returns the name of the middleware.
    /// This should match the name of the property in the containing
    /// `middleware` section in configuration (as named by `serde`)
    fn name(&self) -> &'static str;

    /// Returns whether the middleware is enabled or not.
    /// If the middleware is switchable, take this value from a configuration
    /// value
    fn is_enabled(&self) -> bool {
        true
    }

    /// Returns middleware config.
    ///
    /// # Errors
    /// when could not convert middleware to [`serde_json::Value`]
    fn config(&self) -> serde_json::Result<serde_json::Value>;

    /// Applies the middleware to the given Axum router and returns the modified
    /// router.
    ///
    /// # Errors
    ///
    /// If there is an issue when adding the middleware to the router.
    fn apply(&self, app: Router<Arc<AppContext>>) -> RResult<Router<Arc<AppContext>>>;
}

#[allow(clippy::unnecessary_lazy_evaluations)]
#[must_use]
pub fn default_middleware_stack(ctx: Arc<AppContext>) -> Vec<Box<dyn MiddlewareLayer>> {
    // Shortened reference to middlewares
    let middlewares = &ctx.config.server.middlewares;

    vec![
        // CORS middleware with a default if none
        Box::new(middlewares.cors.clone().unwrap_or_else(|| cors::Cors {
            enable: false,
            ..Default::default()
        })),
        // Catch Panic middleware with a default if none
        Box::new(
            middlewares
                .catch_panic
                .clone()
                .unwrap_or_else(|| catch_panic::CatchPanic { enable: true }),
        ),
        // Etag middleware with a default if none
        Box::new(
            middlewares
                .etag
                .clone()
                .unwrap_or_else(|| etag::Etag { enable: true }),
        ),
        // Remote IP middleware with a default if none
        Box::new(
            middlewares
                .remote_ip
                .clone()
                .unwrap_or_else(|| remote_ip::RemoteIpMiddleware {
                    enable: false,
                    ..Default::default()
                }),
        ),
        // Compression middleware with a default if none
        Box::new(
            middlewares
                .compression
                .clone()
                .unwrap_or_else(|| compression::Compression { enable: false }),
        ),
        // Timeout Request middleware with a default if none
        Box::new(
            middlewares
                .timeout_request
                .clone()
                .unwrap_or_else(|| timeout::TimeOut {
                    enable: false,
                    ..Default::default()
                }),
        ),
        // Secure Headers middleware with a default if none
        Box::new(middlewares.secure_headers.clone().unwrap_or_else(|| {
            secure_headers::SecureHeader {
                enable: false,
                ..Default::default()
            }
        })),
        // Logger middleware with default logger configuration
        Box::new(logger::new(
            &middlewares
                .logger
                .clone()
                .unwrap_or_else(|| logger::Config { enable: true }),
            ctx.clone(),
        )),
        // Request ID middleware with a default if none
        Box::new(
            middlewares
                .request_id
                .clone()
                .unwrap_or_else(|| request_id::RequestId { enable: true }),
        ),
    ]
}

/// Server middleware configuration structure.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct MiddlewareConfig {
    /// Compression for the response.
    pub compression: Option<compression::Compression>,

    /// Etag cache headers.
    pub etag: Option<etag::Etag>,

    /// Logger and augmenting trace id with request data
    pub logger: Option<logger::Config>,

    /// Catch any code panic and log the error.
    pub catch_panic: Option<catch_panic::CatchPanic>,

    /// Setting a global timeout for requests
    pub timeout_request: Option<timeout::TimeOut>,

    /// CORS configuration
    pub cors: Option<cors::Cors>,

    /// Sets a set of secure headers
    pub secure_headers: Option<secure_headers::SecureHeader>,

    /// Calculates a remote IP based on `X-Forwarded-For` when behind a proxy
    pub remote_ip: Option<remote_ip::RemoteIpMiddleware>,

    /// Request ID
    pub request_id: Option<request_id::RequestId>,
}
