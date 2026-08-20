use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use tokio::{net::TcpSocket, signal};
use tower_http::services::{ServeDir, ServeFile};
use tracing::instrument;

use super::{builder::AppBuilder, context::AppContextTrait};
use crate::{
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

    let result = async {
      let listener = {
        let addr: SocketAddr = format!("{}:{}", config.server.binding, config.server.port).parse()?;

        let socket = if addr.is_ipv4() { TcpSocket::new_v4() } else { TcpSocket::new_v6() }?;

        socket.set_reuseaddr(true)?;

        #[cfg(all(unix, not(target_os = "solaris")))]
        if let Err(e) = socket.set_reuseport(true) {
          tracing::warn!("Failed to set SO_REUSEPORT: {}", e);
        }

        socket.bind(addr)?;
        socket.listen(1024)
      }?;

      let router = Self::router(context.clone()).await?.into_make_service_with_connect_info::<SocketAddr>();

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
          task.run_with_signal(if graceful_shutdown { Some(Self::shutdown_signal) } else { None }).await?;

          Ok::<(), RecorderError>(())
        },
      )?;
      Ok::<_, RecorderError>(())
    }
    .await;
    let closed = Self::close_context(context.as_ref()).await;
    result?;
    closed?;
    Ok(())
  }

  pub(crate) async fn close_context(context: &dyn AppContextTrait) -> RecorderResult<()> {
    let auth_closed = context.auth().close().await;
    let queue_closed = context.task().close().await;
    let media_closed = context.media().shutdown().await;
    let business_closed = context.db().as_ref().clone().close().await;
    auth_closed?;
    queue_closed?;
    media_closed?;
    business_closed?;
    Ok(())
  }

  pub async fn router(context: Arc<dyn AppContextTrait>) -> RecorderResult<Router> {
    let mut router = Router::<Arc<dyn AppContextTrait>>::new();

    let (graphql_c, auth_c, metadata_c, static_c, feeds_c, credential_c) = futures::try_join!(
      controller::graphql::create(context.clone()),
      controller::auth::create(context.clone()),
      controller::metadata::create(context.clone()),
      controller::r#static::create(context.clone()),
      controller::feeds::create(context.clone()),
      controller::credential3rd::create(context.clone())
    )?;

    for c in [graphql_c, auth_c, metadata_c, static_c, feeds_c, credential_c] {
      router = c.apply_to(router);
    }

    // The WebUI bundle is public: the same prebuilt assets must
    // load before any auth client exists, and an independent static host
    // serves them with no credential at all.
    router = router
      .route("/api/{*path}", axum::routing::any(|| async { axum::http::StatusCode::NOT_FOUND }))
      .fallback_service(
        ServeDir::new(std::path::Path::new(context.working_dir()).join("webui"))
          .fallback(ServeFile::new(std::path::Path::new(context.working_dir()).join("webui/index.html"))),
      );

    let middlewares = default_middleware_stack(context.clone());
    for mid in middlewares {
      if mid.is_enabled() {
        router = mid.apply(router)?;
        tracing::info!(name = mid.name(), "+middleware");
      }
    }

    // Preflight runs outside authentication and every response, including a
    // denial, carries the same configured browser boundary.
    router = router.layer(crate::web::middleware::cors::for_auth(context.auth())?);
    let mut router = router.with_state(context.clone());
    if let Some(provider) = context.auth().bundled_provider() {
      router = router.nest("/auth-provider", provider.clone().router());
    }
    Ok(router)
  }

  async fn shutdown_signal() {
    let ctrl_c = async {
      signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
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
