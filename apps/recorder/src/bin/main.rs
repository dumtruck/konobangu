use color_eyre::{self, eyre};
use recorder::app::AppBuilder;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let builder = AppBuilder::from_main_cli(None).await?;

    let app = builder.build().await?;

    app.serve().await?;

    Ok(())
}
