use loco_rs::cli;
use recorder::{app::App, migrations::Migrator};

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    cli::main::<App, Migrator>().await?;
    Ok(())
}
