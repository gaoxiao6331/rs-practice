use clap::Parser;
use kv_store::{new_store, serve};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
#[command(
    name = "kv_server",
    about = "A tiny concurrent in-memory key-value server"
)]
struct Args {
    /// The TCP address the server should bind to.
    #[arg(long, default_value = "127.0.0.1:6379")]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let listener = TcpListener::bind(&args.addr).await?;
    println!("Listening on {}...", args.addr);

    // TODO: add on-disk persistence once the in-memory path is stable.
    serve(listener, new_store(), async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;

    Ok(())
}
