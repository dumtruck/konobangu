use tracing::Level;
use tracing_subscriber::EnvFilter;

pub fn try_init_testing_tracing(level: Level) {
    let crate_name = env!("CARGO_PKG_NAME");
    let level = level.as_str().to_lowercase();
    let filter = EnvFilter::new(format!("{crate_name}[]={level}"))
        .add_directive(format!("mockito[]={level}").parse().unwrap());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
