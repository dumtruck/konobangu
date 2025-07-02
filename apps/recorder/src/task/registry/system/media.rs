use std::sync::Arc;

use quirks_path::Path;
use tracing::instrument;

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    media::EncodeImageOptions,
    task::{AsyncTaskTrait, register_system_task_type},
};

register_system_task_type! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct OptimizeImageTask {
        pub source_path: String,
        pub target_path: String,
        pub format_options: EncodeImageOptions,
    }
}

#[async_trait::async_trait]
impl AsyncTaskTrait for OptimizeImageTask {
    #[instrument(err, skip(ctx))]
    async fn run_async(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        let storage = ctx.storage();

        let source_path = Path::new(&self.source_path);

        let media_service = ctx.media();

        let image_data = storage.read(source_path).await?;

        match self.format_options {
            EncodeImageOptions::Webp(options) => {
                let data = media_service
                    .optimize_image_to_webp(source_path, image_data.to_bytes(), Some(options))
                    .await?;
                storage.write(self.target_path, data).await?;
            }
            EncodeImageOptions::Avif(options) => {
                let data = media_service
                    .optimize_image_to_avif(source_path, image_data.to_bytes(), Some(options))
                    .await?;
                storage.write(self.target_path, data).await?;
            }
            EncodeImageOptions::Jxl(options) => {
                let data = media_service
                    .optimize_image_to_jxl(source_path, image_data.to_bytes(), Some(options))
                    .await?;
                storage.write(self.target_path, data).await?;
            }
        };

        Ok(())
    }
}
