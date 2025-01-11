use axum::response::IntoResponse;
use loco_rs::prelude::*;

async fn current() -> impl IntoResponse {
    ""
}

pub fn routes() -> Routes {
    Routes::new().prefix("/auth").add("/current", get(current))
}
