use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

pub type AppState = Arc<SqlitePool>;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub async fn setup_db(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT FALSE
        )",
    )
    .execute(pool)
    .await
    .expect("Failed to create todos table");
}

pub fn app(pool: SqlitePool) -> Router {
    let state: AppState = Arc::new(pool);
    Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/:id",
            axum::routing::patch(update_todo).delete(delete_todo),
        )
        .with_state(state)
}

async fn list_todos(State(pool): State<AppState>) -> Result<Json<Vec<Todo>>, StatusCode> {
    let todos = sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos ORDER BY id")
        .fetch_all(pool.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(todos))
}

async fn create_todo(
    State(pool): State<AppState>,
    Json(input): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), StatusCode> {
    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, completed) VALUES (?, FALSE) RETURNING id, title, completed",
    )
    .bind(&input.title)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(todo)))
}

async fn update_todo(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let existing = sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let title = input.title.unwrap_or(existing.title);
    let completed = input.completed.unwrap_or(existing.completed);

    let todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET title = ?, completed = ? WHERE id = ? RETURNING id, title, completed",
    )
    .bind(&title)
    .bind(completed)
    .bind(id)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(todo))
}

async fn delete_todo(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(pool.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn test_app() -> Router {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory database");
        setup_db(&pool).await;
        app(pool)
    }

    async fn body_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&body).unwrap()
    }

    fn json_request(method: Method, uri: &str, body: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(uri);
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        builder
            .body(Body::from(body.unwrap_or("").to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn test_list_empty() {
        let app = test_app().await;
        let res = app
            .oneshot(json_request(Method::GET, "/todos", None))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let todos: Vec<Todo> = body_json(res).await;
        assert!(todos.is_empty());
    }

    #[tokio::test]
    async fn test_create() {
        let app = test_app().await;
        let res = app
            .oneshot(json_request(
                Method::POST,
                "/todos",
                Some(r#"{"title":"Buy milk"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let todo: Todo = body_json(res).await;
        assert_eq!(todo.title, "Buy milk");
        assert!(!todo.completed);
        assert_eq!(todo.id, 1);
    }

    #[tokio::test]
    async fn test_create_and_list() {
        let app = test_app().await;

        let res = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/todos",
                Some(r#"{"title":"Task 1"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        let res = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/todos",
                Some(r#"{"title":"Task 2"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        let res = app
            .oneshot(json_request(Method::GET, "/todos", None))
            .await
            .unwrap();
        let todos: Vec<Todo> = body_json(res).await;
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].title, "Task 1");
        assert_eq!(todos[1].title, "Task 2");
    }

    #[tokio::test]
    async fn test_update_title() {
        let app = test_app().await;

        let res = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/todos",
                Some(r#"{"title":"Original"}"#),
            ))
            .await
            .unwrap();
        let created: Todo = body_json(res).await;

        let res = app
            .oneshot(json_request(
                Method::PATCH,
                &format!("/todos/{}", created.id),
                Some(r#"{"title":"Updated"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let updated: Todo = body_json(res).await;
        assert_eq!(updated.title, "Updated");
        assert!(!updated.completed);
    }

    #[tokio::test]
    async fn test_complete_todo() {
        let app = test_app().await;

        let res = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/todos",
                Some(r#"{"title":"Do stuff"}"#),
            ))
            .await
            .unwrap();
        let created: Todo = body_json(res).await;

        let res = app
            .oneshot(json_request(
                Method::PATCH,
                &format!("/todos/{}", created.id),
                Some(r#"{"completed":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let updated: Todo = body_json(res).await;
        assert!(updated.completed);
        assert_eq!(updated.title, "Do stuff");
    }

    #[tokio::test]
    async fn test_delete() {
        let app = test_app().await;

        let res = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/todos",
                Some(r#"{"title":"Delete me"}"#),
            ))
            .await
            .unwrap();
        let created: Todo = body_json(res).await;

        let res = app
            .clone()
            .oneshot(json_request(
                Method::DELETE,
                &format!("/todos/{}", created.id),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app
            .oneshot(json_request(Method::GET, "/todos", None))
            .await
            .unwrap();
        let todos: Vec<Todo> = body_json(res).await;
        assert!(todos.is_empty());
    }

    #[tokio::test]
    async fn test_update_nonexistent() {
        let app = test_app().await;
        let res = app
            .oneshot(json_request(
                Method::PATCH,
                "/todos/999",
                Some(r#"{"title":"Nope"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let app = test_app().await;
        let res = app
            .oneshot(json_request(Method::DELETE, "/todos/999", None))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
