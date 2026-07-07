use askama::Template;
use axum::{
    Form, Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::Html,
};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::errors::AppError;
use crate::models::{CreateTodo, CreateTodoForm, Todo, UpdateTodo};

pub type AppState = Arc<SqlitePool>;

pub async fn list_todos(State(pool): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = fetch_all_todos(&pool).await?;
    Ok(Json(todos))
}

pub async fn create_todo(
    State(pool): State<AppState>,
    body: Result<Json<CreateTodo>, JsonRejection>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let Json(input) = body?;
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
        AppError::Internal(e.to_string())
    })?;
    Ok((StatusCode::CREATED, Json(todo)))
}

pub async fn update_todo(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
    body: Result<Json<UpdateTodo>, JsonRejection>,
) -> Result<Json<Todo>, AppError> {
    let Json(input) = body?;
    let existing = sqlx::query_as::<_, Todo>(
        "SELECT id, title, content, completed, created_at, updated_at FROM todos WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, id, "Failed to fetch todo for update");
        AppError::Internal(e.to_string())
    })?
    .ok_or_else(|| AppError::NotFound("not found".to_string()))?;

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
        AppError::Internal(e.to_string())
    })?;
    Ok(Json(todo))
}

pub async fn delete_todo(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(pool.as_ref())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, id, "Failed to delete todo");
            AppError::Internal(e.to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("not found".to_string()));
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

async fn fetch_all_todos(pool: &SqlitePool) -> Result<Vec<Todo>, AppError> {
    sqlx::query_as::<_, Todo>(
        "SELECT id, title, content, completed, created_at, updated_at FROM todos ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to fetch todos");
        AppError::Internal(e.to_string())
    })
}

fn render_html(template: &impl Template) -> Result<Html<String>, AppError> {
    Ok(Html(template.render().map_err(|e| {
        tracing::error!(error = %e, "Failed to render template");
        AppError::Internal(e.to_string())
    })?))
}

pub async fn index_page(State(pool): State<AppState>) -> Result<Html<String>, AppError> {
    let todos = fetch_all_todos(pool.as_ref()).await?;
    render_html(&IndexTemplate { todos })
}

pub async fn create_todo_html(
    State(pool): State<AppState>,
    Form(input): Form<CreateTodoForm>,
) -> Result<Html<String>, AppError> {
    sqlx::query("INSERT INTO todos (title, content, completed) VALUES (?, '', FALSE)")
        .bind(&input.title)
        .execute(pool.as_ref())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create todo (HTML)");
            AppError::Internal(e.to_string())
        })?;

    let todos = fetch_all_todos(pool.as_ref()).await?;
    render_html(&TodoListTemplate { todos })
}

pub async fn toggle_todo_html(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET completed = NOT completed, updated_at = CURRENT_TIMESTAMP WHERE id = ? RETURNING id, title, content, completed, created_at, updated_at",
    )
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, id, "Failed to toggle todo");
        AppError::Internal(e.to_string())
    })?
    .ok_or_else(|| AppError::NotFound("not found".to_string()))?;

    render_html(&TodoItemTemplate { todo })
}

pub async fn delete_todo_html(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(pool.as_ref())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, id, "Failed to delete todo (HTML)");
            AppError::Internal(e.to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("not found".to_string()));
    }
    Ok(Html(String::new()))
}
