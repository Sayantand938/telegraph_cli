mod activities;
mod cli;
mod db;
mod error;
mod transactions;

use clap::Parser;

use cli::Cli;
use db::{connect_db, init_tables};
use transactions::handle_transaction;
use activities::handle_activity;

#[tokio::main]
async fn main() -> error::AppResult<()> {
    let cli = Cli::parse();

    let pool = connect_db().await?;

    init_tables(&pool).await?;

    match cli.domain.as_str() {
        "transaction" => {
            handle_transaction(
                &pool,
                &cli.domain,
                &cli.tool,
                cli.amount,
                cli.kind,
                cli.desc,
                cli.id,
            )
            .await?;
        }
        "activity" => {
            handle_activity(
                &pool,
                &cli.domain,
                &cli.tool,
                cli.start,
                cli.stop,
                cli.activity_desc,
                cli.id,
            )
            .await?;
        }
        _ => return Err(anyhow::anyhow!("Unknown domain: {}", cli.domain)),
    }

    Ok(())
}
