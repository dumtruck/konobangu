use std::{borrow::Cow, sync::Arc};

use axum::Router;

use crate::app::AppContext;

pub trait ControllerTrait: Sized {
    fn apply_to(self, router: Router<Arc<AppContext>>) -> Router<Arc<AppContext>>;
}

pub struct PrefixController {
    prefix: Cow<'static, str>,
    router: Router<Arc<AppContext>>,
}

impl PrefixController {
    pub fn new(prefix: impl Into<Cow<'static, str>>, router: Router<Arc<AppContext>>) -> Self {
        Self {
            prefix: prefix.into(),
            router,
        }
    }
}

impl ControllerTrait for PrefixController {
    fn apply_to(self, router: Router<Arc<AppContext>>) -> Router<Arc<AppContext>> {
        router.nest(&self.prefix, self.router)
    }
}

pub enum Controller {
    Prefix(PrefixController),
}

impl Controller {
    pub fn from_prefix(
        prefix: impl Into<Cow<'static, str>>,
        router: Router<Arc<AppContext>>,
    ) -> Self {
        Self::Prefix(PrefixController::new(prefix, router))
    }
}

impl ControllerTrait for Controller {
    fn apply_to(self, router: Router<Arc<AppContext>>) -> Router<Arc<AppContext>> {
        match self {
            Self::Prefix(p) => p.apply_to(router),
        }
    }
}
