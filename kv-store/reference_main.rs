use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[path = "reference.rs"]
mod reference;

/// KV 参考答案的可运行入口。
///
/// 这个文件负责把 `reference.rs` 里的完整实现接成一个可直接运行的服务端程序。
#[derive(Debug, Parser)]
#[command(
    name = "kv_server_reference",
    about = "Runnable reference solution for the KV server"
)]
struct Args {
    /// 服务监听地址。
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
    let listener = tokio::net::TcpListener::bind(&args.addr).await?;

    println!("Listening on {}...", args.addr);
    reference::serve(listener, reference::new_store(), async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;

    Ok(())
}
