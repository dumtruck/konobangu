use std::sync::Arc;

use async_graphql::{Context, InputObject, Object, Result as GraphQLResult};

use crate::{app::AppContextTrait, auth::AuthUserInfo};

struct TaskQuery;

#[derive(InputObject)]
struct SubscriberTasksFilterInput {
    pub subscription_id: Option<i32>,
    pub task_id: Option<String>,
    pub task_type: Option<String>,
}

#[Object]
impl TaskQuery {
    async fn subscriber_tasks(&self, ctx: &Context<'_>) -> GraphQLResult<Vec<String>> {
        let auth_user_info = ctx.data::<AuthUserInfo>()?;
        let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;
        let subscriber_id = auth_user_info.subscriber_auth.subscriber_id;

        let task_service = app_ctx.task();

        todo!()
    }
}
