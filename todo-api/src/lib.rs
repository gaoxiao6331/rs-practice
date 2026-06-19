use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: Arc<SqlitePool>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq)]
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

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

pub async fn build_state(database_url: &str) -> Result<AppState> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    init_db(&pool).await?;
    Ok(AppState {
        pool: Arc::new(pool),
    })
}

pub async fn init_db(pool: &SqlitePool) -> Result<()> {
    // TODO: move from dynamic queries to `query!` + offline metadata when the schema stabilizes.
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/todos", post(create_todo).get(list_todos))
        .route("/todos/{id}", get(get_todo))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

fn not_implemented(message: &str) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorBody {
            error: message.to_string(),
        }),
    )
}

async fn create_todo(
    State(_state): State<AppState>,
    _payload: Result<Json<CreateTodo>, JsonRejection>,
) -> impl IntoResponse {
    // TODO: 手工实现 DTO 校验、实体转换和数据库写入。
    not_implemented("TODO: implement create_todo by hand")
}

async fn list_todos(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: 手工实现查询列表与返回 JSON 数组。
    not_implemented("TODO: implement list_todos by hand")
}

async fn get_todo(Path(_id): Path<i64>, State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: 手工实现按 ID 查询与 404 错误映射。
    not_implemented("TODO: implement get_todo by hand")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

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
    async fn todo_routes_return_placeholder_before_manual_implementation() {
        let app = app(test_state().await);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/todos")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"学习所有权","completed":false}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    #[ignore = "等待手工补全统一错误处理与查询逻辑"]
    async fn returns_not_found_for_missing_id() {
        let _app = app(test_state().await);
    }
}
