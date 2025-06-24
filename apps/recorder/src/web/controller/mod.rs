pub mod core;
pub mod feeds;
pub mod graphql;
pub mod metadata;
pub mod oidc;
pub mod r#static;

pub use core::{Controller, ControllerTrait, NestRouterController};
