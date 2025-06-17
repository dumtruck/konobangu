use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use tokio::{net::TcpSocket, signal};
use tracing::instrument;

use super::{builder::AppBuilder, context::AppContextTrait};
use crate::{
    app::Environment,
    errors::{RecorderError, RecorderResult},
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

        let (graphql_c, oidc_c, metadata_c, static_c) = futures::try_join!(
            controller::graphql::create(context.clone()),
            controller::oidc::create(context.clone()),
            controller::metadata::create(context.clone()),
            controller::r#static::create(context.clone()),
        )?;

        for c in [graphql_c, oidc_c, metadata_c, static_c] {
            router = c.apply_to(router);
        }

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
        tokio::try_join!(
            async {
                axum::serve(listener, router)
                    .with_graceful_shutdown(async move {
                        Self::shutdown_signal().await;
                        tracing::info!("axum shutting down...");
                    })
                    .await?;
                Ok::<(), RecorderError>(())
            },
            async {
                {
                    let monitor = task.setup_monitor().await?;
                    if matches!(context.environment(), Environment::Development) {
                        monitor.run().await?;
                    } else {
                        monitor
                            .run_with_signal(async move {
                                Self::shutdown_signal().await;
                                tracing::info!("apalis shutting down...");
                                Ok(())
                            })
                            .await?;
                    }
                }

                Ok::<(), RecorderError>(())
            },
            async {
                let listener = task.setup_listener().await?;
                listener.listen().await?;

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

        #[cfg(all(not(unix), debug_assertions))]
        let quit = std::future::pending::<()>();

        tokio::select! {
            () = ctrl_c => {},
            () = terminate => {},
            () = quit => {},
        }
    }
}
