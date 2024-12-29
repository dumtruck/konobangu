pub mod bytes;
pub mod client;
pub mod core;
pub mod html;
pub mod image;

pub use core::DEFAULT_HTTP_CLIENT_USER_AGENT;

pub use bytes::download_bytes;
pub use client::{HttpClient, HttpClientConfig};
pub use image::download_image;
