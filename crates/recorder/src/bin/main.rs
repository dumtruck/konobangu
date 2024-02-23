use loco_rs::cli;
use recorder::migrations::Migrator;
use recorder::app::App;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    cli::main::<App, Migrator>().await
}
