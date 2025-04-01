use recorder::{app::AppBuilder, errors::RResult};

#[tokio::main]
async fn main() -> RResult<()> {
    let builder = AppBuilder::from_main_cli(None).await?;

    let app = builder.build().await?;

    app.serve().await?;

    Ok(())
}
