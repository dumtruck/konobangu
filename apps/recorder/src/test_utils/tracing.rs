use tracing::Level;
use tracing_subscriber::EnvFilter;

pub fn init_testing_tracing(level: Level) {
    let crate_name = env!("CARGO_PKG_NAME");
    let filter = EnvFilter::new(format!(
        "{}[]={}",
        crate_name,
        level.as_str().to_lowercase()
    ));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
