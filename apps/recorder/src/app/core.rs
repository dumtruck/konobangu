use std::sync::Arc;

use super::{builder::AppBuilder, context::AppContext, router::AppRouter};

pub struct App {
    pub context: Arc<AppContext>,
    pub builder: AppBuilder,
    pub router: AppRouter,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }
}
