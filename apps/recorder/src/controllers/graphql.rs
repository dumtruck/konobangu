use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    middleware::from_extractor_with_state,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use loco_rs::{app::AppContext, prelude::Routes};

use crate::{app::AppContextExt, auth::AuthUserInfo};

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new(
        "/api/graphql",
    )))
}

async fn graphql_handler(
    State(ctx): State<AppContext>,
    auth_user_info: AuthUserInfo,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let graphql_service = ctx.get_graphql_service();
    let mut req = req.into_inner();
    req = req.data(auth_user_info);

    graphql_service.schema.execute(req).await.into()
}

pub fn routes(state: AppContext) -> Routes {
    Routes::new()
        .prefix("/graphql")
        .add("/playground", get(graphql_playground))
        .add(
            "/",
            post(graphql_handler)
                .layer(from_extractor_with_state::<AuthUserInfo, AppContext>(state)),
        )
}
