use rs_practice::todo_api::{app, build_state};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = build_state("sqlite://todo.db").await?;
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Todo API listening on http://127.0.0.1:3000");
    axum::serve(listener, app(state)).await?;
    Ok(())
}
