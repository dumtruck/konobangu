use std::borrow::Cow;

use async_trait::async_trait;

use crate::{app::AppContext, errors::RResult};

pub struct TaskVars {}

#[async_trait]
pub trait Task: Send + Sync {
    fn task_name() -> Cow<'static, str>;

    fn task_id(&self) -> &str;

    async fn run(&self, app_context: &AppContext, vars: &TaskVars) -> RResult<()>;
}
