use tracing::Level;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_tree::HierarchicalLayer;

use crate::logger::MODULE_WHITELIST;

fn build_testing_tracing_filter(level: Level) -> EnvFilter {
    let crate_name = env!("CARGO_PKG_NAME");
    let level = level.as_str().to_lowercase();
    let mut filter = EnvFilter::new(format!("{crate_name}[]={level}"));

    let mut modules = vec!["mockito"];
    modules.extend(MODULE_WHITELIST.iter());
    for module in modules {
        filter = filter.add_directive(format!("{module}[]={level}").parse().unwrap());
    }

    filter
}

pub fn try_init_testing_tracing(level: Level) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(build_testing_tracing_filter(level))
        .try_init();
}

pub fn try_init_testing_tracing_only_leaf(level: Level) {
    let _ = tracing_subscriber::registry()
        .with(build_testing_tracing_filter(level))
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .try_init();
}
