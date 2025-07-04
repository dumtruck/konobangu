use std::{net::SocketAddr, sync::Arc};

use axum::{Router, middleware::from_fn_with_state};
use tokio::{net::TcpSocket, signal};
use tower_http::services::{ServeDir, ServeFile};
use tracing::instrument;

use super::{builder::AppBuilder, context::AppContextTrait};
use crate::{
    auth::webui_auth_middleware,
    errors::{RecorderError, RecorderResult},
    web::{
        controller::{self, core::ControllerTrait},
        middleware::default_middleware_stack,
    },
};

pub const PROJECT_NAME: &str = "konobangu";

pub struct App {
    pub context: Arc<dyn AppContextTrait>,
    pub builder: AppBuilder,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    #[instrument(err, skip(self))]
    pub async fn serve(&self) -> RecorderResult<()> {
        let context = &self.context;
        let config = context.config();

        let listener = {
            let addr: SocketAddr =
                format!("{}:{}", config.server.binding, config.server.port).parse()?;

            let socket = if addr.is_ipv4() {
                TcpSocket::new_v4()
            } else {
                TcpSocket::new_v6()
            }?;

            socket.set_reuseaddr(true)?;

            #[cfg(all(unix, not(target_os = "solaris")))]
            if let Err(e) = socket.set_reuseport(true) {
                tracing::warn!("Failed to set SO_REUSEPORT: {}", e);
            }

            socket.bind(addr)?;
            socket.listen(1024)
        }?;

        let mut router = Router::<Arc<dyn AppContextTrait>>::new();

        let (graphql_c, oidc_c, metadata_c, static_c, feeds_c) = futures::try_join!(
            controller::graphql::create(context.clone()),
            controller::oidc::create(context.clone()),
            controller::metadata::create(context.clone()),
            controller::r#static::create(context.clone()),
            controller::feeds::create(context.clone())
        )?;

        for c in [graphql_c, oidc_c, metadata_c, static_c, feeds_c] {
            router = c.apply_to(router);
        }

        router = router
            .fallback_service(
                ServeDir::new("webui").not_found_service(ServeFile::new("webui/index.html")),
            )
            .layer(from_fn_with_state(context.clone(), webui_auth_middleware));

        let middlewares = default_middleware_stack(context.clone());
        for mid in middlewares {
            if mid.is_enabled() {
                router = mid.apply(router)?;
                tracing::info!(name = mid.name(), "+middleware");
            }
        }

        let router = router
            .with_state(context.clone())
            .into_make_service_with_connect_info::<SocketAddr>();

        let task = context.task();

        let graceful_shutdown = self.builder.graceful_shutdown;

        tokio::try_join!(
            async {
                let axum_serve = axum::serve(listener, router);

                if graceful_shutdown {
                    axum_serve
                        .with_graceful_shutdown(async move {
                            Self::shutdown_signal().await;
                            tracing::info!("axum shutting down...");
                        })
                        .await?;
                } else {
                    axum_serve.await?;
                }

                Ok::<(), RecorderError>(())
            },
            async {
                task.run_with_signal(if graceful_shutdown {
                    Some(Self::shutdown_signal)
                } else {
                    None
                })
                .await?;

                Ok::<(), RecorderError>(())
            }
        )?;

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

        #[cfg(all(unix, debug_assertions))]
        let quit = async {
            signal::unix::signal(signal::unix::SignalKind::quit())
                .expect("Failed to install SIGQUIT handler")
                .recv()
                .await;
            println!("Received SIGQUIT");
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        #[cfg(not(all(unix, debug_assertions)))]
        let quit = std::future::pending::<()>();

        tokio::select! {
            () = ctrl_c => {},
            () = terminate => {},
            () = quit => {},
        }
    }
}
