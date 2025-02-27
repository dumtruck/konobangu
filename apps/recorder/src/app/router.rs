use std::sync::Arc;

use axum::Router;
use futures::try_join;

use crate::{
    app::AppContext,
    controllers::{self, core::ControllerTrait},
    errors::RResult,
};

pub struct AppRouter {
    pub root: Router<Arc<AppContext>>,
}

pub async fn create_router(context: Arc<AppContext>) -> RResult<AppRouter> {
    let mut root_router = Router::<Arc<AppContext>>::new();

    let (graphqlc, oidcc) = try_join!(
        controllers::graphql::create(context.clone()),
        controllers::oidc::create(context.clone()),
    )?;

    for c in [graphqlc, oidcc] {
        root_router = c.apply_to(root_router);
    }

    root_router = root_router.with_state(context);

    Ok(AppRouter { root: root_router })
}
