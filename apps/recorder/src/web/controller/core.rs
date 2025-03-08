use std::{borrow::Cow, sync::Arc};

use axum::Router;

use crate::app::AppContextTrait;

pub trait ControllerTrait: Sized {
    fn apply_to(self, router: Router<Arc<dyn AppContextTrait>>)
    -> Router<Arc<dyn AppContextTrait>>;
}

pub struct PrefixController {
    prefix: Cow<'static, str>,
    router: Router<Arc<dyn AppContextTrait>>,
}

impl PrefixController {
    pub fn new(
        prefix: impl Into<Cow<'static, str>>,
        router: Router<Arc<dyn AppContextTrait>>,
    ) -> Self {
        Self {
            prefix: prefix.into(),
            router,
        }
    }
}

impl ControllerTrait for PrefixController {
    fn apply_to(
        self,
        router: Router<Arc<dyn AppContextTrait>>,
    ) -> Router<Arc<dyn AppContextTrait>> {
        router.nest(&self.prefix, self.router)
    }
}

pub enum Controller {
    Prefix(PrefixController),
}

impl Controller {
    pub fn from_prefix(
        prefix: impl Into<Cow<'static, str>>,
        router: Router<Arc<dyn AppContextTrait>>,
    ) -> Self {
        Self::Prefix(PrefixController::new(prefix, router))
    }
}

impl ControllerTrait for Controller {
    fn apply_to(
        self,
        router: Router<Arc<dyn AppContextTrait>>,
    ) -> Router<Arc<dyn AppContextTrait>> {
        match self {
            Self::Prefix(p) => p.apply_to(router),
        }
    }
}
