use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().fallback_service(ServeDir::new("markdown-wasm"));
    let listener = TcpListener::bind("127.0.0.1:4000").await?;

    println!("Serving markdown demo on http://127.0.0.1:4000");
    println!("Run `wasm-pack build --target web --out-dir markdown-wasm/pkg` first.");
    axum::serve(listener, app).await?;
    Ok(())
}
