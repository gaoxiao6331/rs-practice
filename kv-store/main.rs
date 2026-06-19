use clap::Parser;
use rs_practice::kv_store::new_store;
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
#[command(name = "kv_server", about = "A minimal kv-server skeleton")]
struct Args {
    #[arg(long, default_value = "127.0.0.1:6379")]
    addr: String,
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    runtime.block_on(async_main());
}

async fn async_main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let _store = new_store();
    let listener = TcpListener::bind(&args.addr)
        .await
        .expect("failed to bind tcp listener");

    println!("Listening on {}...", args.addr);
    info!("kv server skeleton started");

    loop {
        match listener.accept().await {
            Ok((_socket, peer)) => {
                warn!(%peer, "TODO: hand-write command parsing and shared-state access");
            }
            Err(error) => {
                warn!(%error, "accept failed");
            }
        }
    }
}
