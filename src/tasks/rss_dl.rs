use std::collections::BTreeMap;

use loco_rs::prelude::*;

pub struct RssDl;
#[async_trait]
impl Task for RssDl {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "rss_dl".to_string(),
            detail: "Task generator".to_string(),
        }
    }
    async fn run(&self, _app_context: &AppContext, _vars: &BTreeMap<String, String>) -> Result<()> {
        println!("Task RssDl generated");
        Ok(())
    }
}
