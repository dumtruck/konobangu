use loco_rs::prelude::*;
use secrecy::SecretString;

#[derive(Clone, Debug)]
pub struct CreateMikanRSSFromMyBangumiTask {
    subscriber_id: i32,
    task_id: String,
    cookie: SecretString,
}

#[async_trait::async_trait]
impl Task for CreateMikanRSSFromMyBangumiTask {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: format!(
                "create-mikan-rss-from-my-bangumi-{}-{}",
                self.subscriber_id, self.task_id
            ),
            detail: "create mikan rss from my bangumi page for {} {}".to_string(),
        }
    }

    async fn run(&self, _app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        Ok(())
    }
}
