use askama::Template;
use axum::{
    Form, Json,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::{CreateTodo, CreateTodoForm, Todo, UpdateTodo};

pub type AppState = Arc<SqlitePool>;

pub async fn list_todos(State(pool): State<AppState>) -> Result<Json<Vec<Todo>>, StatusCode> {
    let todos = fetch_todos(&pool).await?;
    Ok(Json(todos))
}

pub async fn create_todo(
    State(pool): State<AppState>,
    Json(input): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), StatusCode> {
    let content = input.content.unwrap_or_default();
    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, content, completed) VALUES (?, ?, FALSE) RETURNING id, title, content, completed, created_at, updated_at",
    )
    .bind(&input.title)
    .bind(&content)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to create todo");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok((StatusCode::CREATED, Json(todo)))
}

pub async fn update_todo(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let existing = sqlx::query_as::<_, Todo>(
        "SELECT id, title, content, completed, created_at, updated_at FROM todos WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, id, "Failed to fetch todo for update");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let title = input.title.unwrap_or(existing.title);
    let content = input.content.unwrap_or(existing.content);
    let completed = input.completed.unwrap_or(existing.completed);

    let todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET title = ?, content = ?, completed = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? RETURNING id, title, content, completed, created_at, updated_at",
    )
    .bind(&title)
    .bind(&content)
    .bind(completed)
    .bind(id)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, id, "Failed to update todo");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
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
        .map_err(|e| {
            tracing::error!(error = %e, id, "Failed to delete todo");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---- HTML/HTMX Handlers ----

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    todos: Vec<Todo>,
}

#[derive(Template)]
#[template(path = "todo_list.html")]
struct TodoListTemplate {
    todos: Vec<Todo>,
}

#[derive(Template)]
#[template(path = "todo_item.html")]
struct TodoItemTemplate {
    todo: Todo,
}

async fn fetch_todos(pool: &SqlitePool) -> Result<Vec<Todo>, StatusCode> {
    sqlx::query_as::<_, Todo>(
        "SELECT id, title, content, completed, created_at, updated_at FROM todos ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to fetch todos");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

fn render_html(template: &impl Template) -> Result<Html<String>, StatusCode> {
    Ok(Html(
        template
            .render()
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to render template");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
    ))
}

pub async fn index_page(State(pool): State<AppState>) -> Result<Html<String>, StatusCode> {
    let todos = fetch_todos(pool.as_ref()).await?;
    render_html(&IndexTemplate { todos })
}

pub async fn create_todo_html(
    State(pool): State<AppState>,
    Form(input): Form<CreateTodoForm>,
) -> Result<Html<String>, StatusCode> {
    sqlx::query("INSERT INTO todos (title, content, completed) VALUES (?, '', FALSE)")
        .bind(&input.title)
        .execute(pool.as_ref())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create todo (HTML)");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let todos = fetch_todos(pool.as_ref()).await?;
    render_html(&TodoListTemplate { todos })
}

pub async fn toggle_todo_html(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET completed = NOT completed, updated_at = CURRENT_TIMESTAMP WHERE id = ? RETURNING id, title, content, completed, created_at, updated_at",
    )
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, id, "Failed to toggle todo");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    render_html(&TodoItemTemplate { todo })
}

pub async fn delete_todo_html(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(pool.as_ref())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, id, "Failed to delete todo (HTML)");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Html(String::new()))
}
