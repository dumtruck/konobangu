use recorder::{app::AppBuilder, errors::RecorderResult};

#[tokio::main]
async fn main() -> RecorderResult<()> {
    let builder = AppBuilder::from_main_cli(None).await?;

    let app = builder.build().await?;

    app.serve().await?;

    Ok(())
}
