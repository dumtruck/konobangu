use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::GraphQL;
use axum::{
    response::{Html, IntoResponse},
    routing::{get, post_service},
};
use loco_rs::prelude::Routes;

use crate::graphql::service::AppGraphQLService;

pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new(
        "/api/graphql",
    )))
}

pub fn routes(graphql_service: &AppGraphQLService) -> Routes {
    Routes::new()
        .prefix("/graphql")
        .add("/playground", get(graphql_playground))
        .add(
            "/",
            post_service(GraphQL::new(graphql_service.schema.clone())),
        )
}
