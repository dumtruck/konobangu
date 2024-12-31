pub mod bytes;
pub mod client;
pub mod core;
pub mod html;
pub mod image;

pub use core::DEFAULT_HTTP_CLIENT_USER_AGENT;

pub use bytes::fetch_bytes;
pub use client::{HttpClient, HttpClientConfig};
pub use html::fetch_html;
pub use image::fetch_image;
