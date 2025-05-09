use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::logger::MODULE_WHITELIST;

pub fn try_init_testing_tracing(level: Level) {
    let crate_name = env!("CARGO_PKG_NAME");
    let level = level.as_str().to_lowercase();
    let mut filter = EnvFilter::new(format!("{crate_name}[]={level}"));

    let mut modules = vec![];
    modules.extend(MODULE_WHITELIST.iter());
    modules.push("mockito");
    for module in modules {
        filter = filter.add_directive(format!("{module}[]={level}").parse().unwrap());
    }

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
