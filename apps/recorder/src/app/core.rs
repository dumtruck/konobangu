use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use futures::try_join;
use tokio::signal;

use super::{builder::AppBuilder, context::AppContextTrait};
use crate::{
    errors::RResult,
    web::{
        controller::{self, core::ControllerTrait},
        middleware::default_middleware_stack,
    },
};

pub struct App {
    pub context: Arc<dyn AppContextTrait>,
    pub builder: AppBuilder,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub async fn serve(&self) -> RResult<()> {
        let context = &self.context;
        let config = context.config();
        let listener = tokio::net::TcpListener::bind(&format!(
            "{}:{}",
            config.server.binding, config.server.port
        ))
        .await?;

        let mut router = Router::<Arc<dyn AppContextTrait>>::new();

        let (graphql_c, oidc_c, metadata_c) = try_join!(
            controller::graphql::create(context.clone()),
            controller::oidc::create(context.clone()),
            controller::metadata::create(context.clone())
        )?;

        for c in [graphql_c, oidc_c, metadata_c] {
            router = c.apply_to(router);
        }

        let middlewares = default_middleware_stack(context.clone());
        for mid in middlewares {
            router = mid.apply(router)?;
            tracing::info!(name = mid.name(), "+middleware");
        }

        let router = router
            .with_state(context.clone())
            .into_make_service_with_connect_info::<SocketAddr>();

        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                Self::shutdown_signal().await;
                tracing::info!("shutting down...");
            })
            .await?;

        Ok(())
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            () = ctrl_c => {},
            () = terminate => {},
        }
    }
}
