use loco_rs::cli;
use recorder::{app::App, migrations::Migrator};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    cli::main::<App, Migrator>().await?;
    Ok(())
}
