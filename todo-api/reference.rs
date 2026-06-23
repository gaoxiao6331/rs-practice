use std::sync::Arc;

use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};

/// Todo API 的参考答案版本。
///
/// 这份实现覆盖了：
/// - SQLite 初始化
/// - `POST /todos`
/// - `GET /todos`
/// - `GET /todos/{id}`
/// - 统一错误响应
///
/// 同时它也演示了 requirement 里提到的几个重点概念：
/// - DTO / DO（这里直接用 `CreateTodo` 和 `Todo`）
/// - `AppError -> IntoResponse`
/// - `axum` 路由与提取器

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

/// 统一错误类型。
///
/// 好处是：路由函数只需要返回 `Result<T, AppError>`，
/// 最后的 HTTP 映射都统一收敛到一个地方。
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(i64),
    Database(sqlx::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError::Database(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::NotFound(id) => (StatusCode::NOT_FOUND, format!("todo {id} not found")),
            AppError::Database(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}

/// 连接数据库并初始化表结构。
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

/// 执行 migrations。
pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("todo-api/migrations").run(pool).await?;
    Ok(())
}

/// 组装路由。
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

/// 创建 Todo。
///
/// 这里故意把 `Json<CreateTodo>` 写成 `Result<Json<CreateTodo>, JsonRejection>`，
/// 这样我们就可以把框架默认的 JSON 解析错误转成自定义的统一错误格式。
async fn create_todo(
    State(state): State<AppState>,
    payload: Result<Json<CreateTodo>, JsonRejection>,
) -> Result<impl IntoResponse, AppError> {
    let payload = payload
        .map_err(|error| AppError::BadRequest(error.body_text()))?
        .0;

    if payload.title.trim().is_empty() {
        return Err(AppError::BadRequest("title cannot be empty".to_string()));
    }

    let title = payload.title.trim().to_string();

    let result = sqlx::query("INSERT INTO todos (title, completed) VALUES (?, ?)")
        .bind(&title)
        .bind(payload.completed)
        .execute(state.pool.as_ref())
        .await?;

    let todo = Todo {
        id: result.last_insert_rowid(),
        title,
        completed: payload.completed,
    };

    Ok((StatusCode::CREATED, Json(todo)))
}

async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos ORDER BY id ASC")
        .fetch_all(state.pool.as_ref())
        .await?;

    Ok(Json(todos))
}

async fn get_todo(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Todo>, AppError> {
    let todo = sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos WHERE id = ?")
        .bind(id)
        .fetch_optional(state.pool.as_ref())
        .await?;

    todo.map(Json).ok_or(AppError::NotFound(id))
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
    async fn create_and_list_todos() {
        let app = app(test_state().await);

        let response = app
            .clone()
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

        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/todos")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_json_returns_bad_request() {
        let app = app(test_state().await);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/todos")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"broken","completed":"oops"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn missing_id_returns_not_found() {
        let app = app(test_state().await);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/todos/404")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
