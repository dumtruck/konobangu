use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::GraphQL;
use axum::response::Html;
use loco_rs::prelude::*;

use crate::graphql::service::AppGraphQLService;

pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new(
        "/api/graphql",
    )))
}

pub fn routes(graphql_service: &AppGraphQLService) -> Routes {
    Routes::new().prefix("/graphql").add(
        "/",
        get(graphql_playground).post_service(GraphQL::new(graphql_service.schema.clone())),
    )
}
