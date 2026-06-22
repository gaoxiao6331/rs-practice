use clap::Parser;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[path = "reference.rs"]
mod reference;

/// Todo API 参考答案的可运行入口。
#[derive(Debug, Parser)]
#[command(
    name = "todo_api_reference",
    about = "Runnable reference solution for the Todo API"
)]
struct Args {
    /// HTTP 监听地址。
    #[arg(long, default_value = "127.0.0.1:3000")]
    addr: String,
    /// SQLite 数据库文件路径。
    #[arg(long, default_value = "todo.db")]
    database: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let database_url = format!("sqlite://{}", args.database);
    let state = reference::build_state(&database_url).await?;
    let listener = TcpListener::bind(&args.addr).await?;

    println!("Todo API listening on http://{}", args.addr);
    axum::serve(listener, reference::app(state)).await?;
    Ok(())
}
