use loco_rs::cli;
use recorder::{app::App, migrations::Migrator, utils::cli::hack_env_to_fit_workspace};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    hack_env_to_fit_workspace()?;
    cli::main::<App, Migrator>().await
}
