//! Loopback test support: the production AppContext/router and the shared
//! PostgreSQL fixture.
use std::path::PathBuf;

use figment::{Figment, providers::Serialized};
use recorder::app::{App, AppConfig, AppContext, AppContextTrait, Environment};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::watch};

#[path = "../../integration/rls/support.rs"]
mod support;

#[derive(Deserialize)]
struct FixtureConfig {
  issuer: String,
  issuer_ca_file: Option<PathBuf>,
  ports: [u16; 5],
  webui_origins: [String; 5],
  alternate_webui_origin: String,
  lan_webui_origin: String,
  lan_bundled_webui_origin: String,
  bundled_issuers: [String; 2],
  bundled_private_keys: [String; 2],
  directory: PathBuf,
}

async fn run_main() -> Result<(), Box<dyn std::error::Error>> {
  tokio::task::LocalSet::new().run_until(run()).await
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let config_path = std::env::var("KONOBANGU_AUTH_TEST_CONFIG")?;
  let fixture_config: FixtureConfig = serde_json::from_slice(&std::fs::read(config_path)?)?;
  let fixture = support::RlsFixture::new().await;
  use sea_orm::ConnectionTrait;
  // Make auth ID differ from subscriber ID to detect private-file identity
  // confusion.
  fixture
    .bootstrap
    .execute_unprepared("INSERT INTO auth_identity.auth (pid, auth_type, subscriber_id) VALUES ('fixture-extra-basic', 'basic', 1001)")
    .await?;
  fixture
    .bootstrap
    .execute_raw(sea_orm::Statement::from_sql_and_values(
      sea_orm::DbBackend::Postgres,
      "INSERT INTO auth_identity.auth (pid, auth_type, issuer, subscriber_id) VALUES ('A','oidc',$1,1001),('B','oidc',$1,1002)",
      [fixture_config.issuer.clone().into()],
    ))
    .await?;
  let base_url = format!("http://127.0.0.1:{}", fixture_config.ports[0]);
  let working_dir = fixture_config.directory.to_string_lossy().into_owned();
  let mut db_config = fixture.bootstrap.config.clone();
  db_config.auto_migrate = true;
  db_config.max_connections = 5;
  let (shutdown, shutdown_rx) = watch::channel(false);
  let mut servers = Vec::new();
  let mut contexts = Vec::new();
  for (index, port) in fixture_config.ports.into_iter().enumerate() {
    let provider = if index == 2 || index == 4 {
      let bundled_index = if index == 2 { 0 } else { 1 };
      let mut callbacks = vec![format!("{}/auth/callback", fixture_config.webui_origins[index])];
      if index == 2 {
        callbacks.push(format!("{}/auth/callback", fixture_config.lan_bundled_webui_origin));
      }
      serde_json::json!({
        "type":"bundled", "issuer":fixture_config.bundled_issuers[bundled_index],
        "client":{"client_id":"konobangu-test","redirect_uris":callbacks},
        "credentials":{"username":"fixture-admin","password":"fixture-admin-password","subject":"local-admin"},
        "signing":{"private_key_pem":fixture_config.bundled_private_keys[bundled_index]},
        "tokens":{"access_token_ttl_seconds":30}
      })
    } else {
      // The browser client is public; the API audience is separate from it and
      // token lifetime belongs to the IdP, so no timing is configured here.
      serde_json::json!({ "type": "oidc", "issuer": fixture_config.issuer, "trusted_ca_file": fixture_config.issuer_ca_file, "browser": { "client_id": "konobangu-test", "scopes": ["openid", "profile", "offline_access", "konobangu:api"] } })
    };
    let mut origins = vec![serde_json::json!({"type":"strict", "value": fixture_config.webui_origins[index]})];
    if index == 1 {
      origins.push(serde_json::json!({"type":"strict", "value": fixture_config.alternate_webui_origin}));
      origins.push(serde_json::json!({"type":"strict", "value": fixture_config.lan_webui_origin}));
    }
    if index == 2 {
      origins.push(serde_json::json!({"type":"strict", "value": fixture_config.lan_bundled_webui_origin}));
    }
    let webui = serde_json::json!({ "allowed_origins": origins });
    let mut identity = serde_json::json!({});
    // One instance also covers optional separate runtime credentials.
    let mut database = db_config.clone();
    let scheduler = if index == 1 {
      database.uri = fixture.app_uri.clone();
      database.auto_migrate = false;
      identity["database_url"] = fixture.identity_uri.clone().into();
      serde_json::json!({"database": {"url": fixture.queue_uri}})
    } else {
      serde_json::json!({})
    };
    let auth = serde_json::json!({ "provider": provider, "webui": webui, "identity": identity });
    let config: AppConfig = Figment::new()
      .merge(AppConfig::default_provider())
      .merge(Serialized::defaults(serde_json::json!({
          "database": database, "auth": auth, "scheduler": scheduler,
          "server": { "host": "127.0.0.1", "port": port, "middlewares": { "logger": { "enable": false }, "secure_headers": { "enable": index == 2, "preset": "github" } } },
          "logger": { "enable": false, "level": "off", "format": "compact" },
          "storage": { "data_dir": fixture_config.directory.join(format!("data-{index}")) }
      })))
      .extract()?;
    let context = AppContext::new(Environment::Testing, config, &working_dir).await?;
    if index == 4 {
      context.graphql().export_schema(&fixture_config.directory.join("schema.graphql"))?;
    }
    if index == 0 {
      fixture.bootstrap.execute_unprepared("INSERT INTO subscriptions (id,display_name,subscriber_id,source_url,enabled,category) VALUES (6000,'Media browser A',1001,'https://example.test/media-a',false,'mikan_subscriber'),(6001,'Media browser B',1002,'https://example.test/media-b',false,'mikan_subscriber'); INSERT INTO bangumi (id,bangumi_type,subscriber_id,display_name,origin_name,season,poster_link) VALUES (6010,'mikan',1001,'Private browser poster A','Private browser poster A',1,'/subscribers/1001/image/fixture/browser-poster.png'),(6011,'mikan',1002,'Private browser poster B','Private browser poster B',1,'/subscribers/1002/image/fixture/browser-poster.png'); INSERT INTO subscription_bangumi (subscriber_id,subscription_id,bangumi_id) VALUES (1001,6000,6010),(1002,6001,6011)").await?;
    }
    if index == 0 {
      fixture.insert_rss_feed(1001, 6000, "browser-feed-A").await;
      fixture.insert_rss_feed(1002, 6001, "browser-feed-B").await;
    }
    for subscriber in [1, 2, 3, 4, 1001, 1002] {
      let private = context.storage().build_subscriber_path(subscriber, "fixture-private.txt");
      context
        .storage()
        .operator
        .write(private.as_ref(), format!("fixture-private-content-{subscriber}"))
        .await?;
    }
    if index == 2 {
      fixture.bootstrap.execute_unprepared("INSERT INTO subscriptions (id,display_name,subscriber_id,source_url,enabled,category) VALUES (5000,'Media browser fixture',1,'https://example.test/media',true,'mikan_subscriber'); INSERT INTO bangumi (id,bangumi_type,subscriber_id,display_name,origin_name,season,poster_link) VALUES (5010,'mikan',1,'Media browser poster','Media browser poster',1,'/subscribers/1/image/fixture/browser-poster.png'); INSERT INTO subscription_bangumi (subscriber_id,subscription_id,bangumi_id) VALUES (1,5000,5010)").await?;
      fixture.insert_rss_feed(1, 5000, "browser-feed-bundled").await;
      fixture
        .bootstrap
        .execute_unprepared(
          "INSERT INTO cron (id,subscriber_id,subscription_id,cron_expr,cron_timezone,next_run,last_run,enabled,subscriber_task_cron) VALUES (5020,1,5000,'0 \
           */5 * * * *','America/New_York','2026-10-04T19:38:05.559928Z','2026-10-04T19:38:05.559928Z',false,'{\"task_type\":\"sync_one_subscription_sources\"\
           ,\"subscription_id\":5000,\"subscriber_id\":1}'::jsonb)",
        )
        .await?;
      fixture
        .bootstrap
        .execute_unprepared(
          "INSERT INTO task_runs (id,kind,subscriber_id,subscription_id,cron_id,payload,task_type,status,attempts,max_attempts,last_error) VALUES \
           ('fixture-cron-run','subscriber_task',1,5000,5020,'{\"task_type\":\"sync_one_subscription_sources\",\"subscription_id\":5000,\"subscriber_id\":1}'\
           ::jsonb,'sync_one_subscription_sources','Failed',3,3,'Fixture previous run failed')",
        )
        .await?;
    }
    if index != 4 {
      let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(256, 256, |x, y| image::Rgba([255, y as u8, 0, x as u8])));
      let mut bytes = std::io::Cursor::new(Vec::new());
      image.write_to(&mut bytes, image::ImageFormat::Png)?;
      let owners = if index == 2 { vec![1] } else { vec![1001, 1002] };
      for subscriber_id in owners {
        let source = format!("/subscribers/{subscriber_id}/image/fixture/browser-poster.png");
        context.storage().write(&source, bytes::Bytes::from(bytes.get_ref().clone())).await?;
        for plan in context.media().derivative_plans() {
          let fingerprint = recorder::media::derivative::fingerprint(&context.storage().stat(&source).await?).unwrap();
          let mut task = recorder::task::OptimizeImageTask::builder()
            .source_path(source.clone())
            .target_path("fixture".into())
            .format_options(plan.options)
            .derivation_version(Some(1))
            .source_fingerprint(Some(fingerprint))
            .build();
          task.subscriber_id = Some(subscriber_id);
          use recorder::task::AsyncTaskTrait;
          task.run_async(context.clone()).await?;
        }
      }
    }
    let network_log = fixture_config.directory.join("media-network.jsonl");
    let router = App::router(context.clone()).await?.layer(axum::middleware::from_fn(move |request: axum::extract::Request, next: axum::middleware::Next| {
      let network_log = network_log.clone();
      async move {
        let path = request.uri().to_string();
        let headers = request.headers();
        let accept = headers.get("accept").and_then(|v| v.to_str().ok()).unwrap_or("").to_owned();
        let ua = headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("").to_owned();
        let response = next.run(request).await;
        if path.contains("browser-poster.png") {
          use std::io::Write;
          let mut log = std::fs::OpenOptions::new().create(true).append(true).open(network_log).expect("fixture network log");
          writeln!(log,"{}",serde_json::json!({"path":path,"accept":accept,"ua":ua,"status":response.status().as_u16(),"content_type":response.headers().get("content-type").and_then(|v|v.to_str().ok()),"content_length":response.headers().get("content-length").and_then(|v|v.to_str().ok()),"etag":response.headers().get("etag").and_then(|v|v.to_str().ok())})).expect("fixture network record");
        }
        response
      }
    }));
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    let mut cleaner_receiver = shutdown_rx.clone();
    servers.push(tokio::spawn(async move {
      let _ = cleaner_receiver.changed().await;
      Ok::<_, std::io::Error>(())
    }));
    let mut receiver = shutdown_rx.clone();
    servers.push(tokio::spawn(async move {
      axum::serve(listener, router.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(async move {
          let _ = receiver.changed().await;
        })
        .await
    }));
    if index == 0 {
      let worker_context = context.clone();
      let mut stopped = shutdown_rx.clone();
      servers.push(tokio::task::spawn_local(async move {
        worker_context
          .task()
          .run_with_signal(Some(|| async move {
            let _ = stopped.changed().await;
          }))
          .await
          .map_err(|_| std::io::Error::other("Task fixture worker failed"))
      }));
    }
    contexts.push(context);
  }
  std::fs::write(
    fixture_config.directory.join("server.json"),
    serde_json::to_vec(&serde_json::json!({
        "baseUrl": base_url,
        "instanceUrl": format!("http://127.0.0.1:{}", fixture_config.ports[1]),
        "bundledUrl": format!("http://127.0.0.1:{}", fixture_config.ports[2]),
        "secureUrl": format!("http://127.0.0.1:{}", fixture_config.ports[3]),
        "expiresUrl": format!("http://127.0.0.1:{}", fixture_config.ports[4]),
        "bundledIssuer": fixture_config.bundled_issuers[0],
        "devIssuer": fixture_config.bundled_issuers[1],
        "schemaPath": fixture_config.directory.join("schema.graphql"),
        "issuer": fixture_config.issuer
    }))?,
  )?;
  #[cfg(unix)]
  {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! { result = tokio::signal::ctrl_c() => result?, _ = terminate.recv() => {} }
  }
  #[cfg(not(unix))]
  tokio::signal::ctrl_c().await?;
  shutdown.send(true)?;
  for server in servers {
    server.await??;
  }
  for context in contexts {
    context.db().as_ref().clone().close().await?;
    context.auth().close().await?;
    context.task().queue_database()?.clone().close().await?;
  }
  fixture.close().await;
  Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // LocalSet cleanup must use the current-thread testcontainers drop worker.
  tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(run_main())
}
