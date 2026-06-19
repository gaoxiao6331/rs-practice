use std::path::PathBuf;

use clap::Parser;
use todo_api::{app, build_state};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
#[command(
    name = "todo_api",
    about = "A minimal RESTful Todo API backed by SQLite"
)]
struct Args {
    /// The HTTP address the API should bind to.
    #[arg(long, default_value = "127.0.0.1:3000")]
    addr: String,
    /// SQLite database file path.
    #[arg(long, default_value = "todo.db")]
    database: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let database_url = format!("sqlite://{}", args.database.display());
    let state = build_state(&database_url).await?;
    let listener = TcpListener::bind(&args.addr).await?;

    println!("Todo API listening on http://{}", args.addr);
    axum::serve(listener, app(state)).await?;
    Ok(())
}
