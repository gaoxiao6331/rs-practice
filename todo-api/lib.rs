use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTodo {
    pub title: String,
    pub completed: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<SqlitePool>,
}

pub async fn build_state(database_url: &str) -> Result<AppState, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    init_db(&pool).await?;
    Ok(AppState {
        pool: Arc::new(pool),
    })
}

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("todo-api/migrations").run(pool).await?;
    Ok(())
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/todos", post(not_implemented).get(not_implemented))
        .route("/todos/{id}", get(not_implemented))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn not_implemented() -> impl IntoResponse {
    // TODO: 手工实现领域模型转换、AppError 和 CRUD 路由逻辑。
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "TODO: implement todo api by hand"
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    async fn test_state() -> AppState {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        init_db(&pool).await.unwrap();
        AppState {
            pool: Arc::new(pool),
        }
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let app = app(test_state().await);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn todo_routes_are_placeholder_routes() {
        let app = app(test_state().await);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/todos")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }
}
