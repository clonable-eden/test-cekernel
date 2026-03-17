use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::{CreateTodo, Todo, UpdateTodo};

pub type AppState = Arc<SqlitePool>;

pub async fn list_todos(State(pool): State<AppState>) -> Result<Json<Vec<Todo>>, StatusCode> {
    let todos = sqlx::query_as::<_, Todo>(
        "SELECT id, title, completed, created_at, updated_at FROM todos ORDER BY id",
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(todos))
}

pub async fn create_todo(
    State(pool): State<AppState>,
    Json(input): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), StatusCode> {
    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, completed) VALUES (?, FALSE) RETURNING id, title, completed, created_at, updated_at",
    )
    .bind(&input.title)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(todo)))
}

pub async fn update_todo(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let existing = sqlx::query_as::<_, Todo>(
        "SELECT id, title, completed, created_at, updated_at FROM todos WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let title = input.title.unwrap_or(existing.title);
    let completed = input.completed.unwrap_or(existing.completed);

    let todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET title = ?, completed = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? RETURNING id, title, completed, created_at, updated_at",
    )
    .bind(&title)
    .bind(completed)
    .bind(id)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(todo))
}

pub async fn delete_todo(
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
