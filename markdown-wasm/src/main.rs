use axum::Router;
use clap::Parser;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[derive(Debug, Parser)]
#[command(
    name = "markdown_preview",
    about = "Serve the WASM markdown demo files"
)]
struct Args {
    /// The HTTP address used to serve index.html and the generated pkg directory.
    #[arg(long, default_value = "127.0.0.1:4000")]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let app = Router::new().fallback_service(ServeDir::new("."));
    let listener = TcpListener::bind(&args.addr).await?;

    println!("Serving markdown demo on http://{}", args.addr);
    println!("Run `wasm-pack build --target web --out-dir pkg` before opening the page.");
    // TODO: auto-run wasm-pack when it is available in the local toolchain.
    axum::serve(listener, app).await?;
    Ok(())
}
