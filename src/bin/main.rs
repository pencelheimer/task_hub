use loco_rs::cli;
use migration::Migrator;
use task_hub::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App, Migrator>().await
}
