pub mod errors;
pub mod handlers;
pub mod models;

pub use errors::{AppError, ErrorResponse};
pub use handlers::AppState;
pub use models::{CreateTodo, CreateTodoForm, Todo, UpdateTodo};

use axum::{Router, routing::get};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use handlers::{
    create_todo, create_todo_html, delete_todo, delete_todo_html, index_page, list_todos,
    toggle_todo_html, update_todo,
};

pub async fn setup_db(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            completed BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool)
    .await
    .expect("Failed to create todos table");
}

pub fn app(pool: SqlitePool) -> Router {
    let state: AppState = Arc::new(pool);
    Router::new()
        .route("/", get(index_page).post(create_todo_html))
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/{id}",
            axum::routing::patch(update_todo).delete(delete_todo),
        )
        .route("/todos/{id}/toggle", axum::routing::post(toggle_todo_html))
        .route("/todos/{id}/delete", axum::routing::post(delete_todo_html))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
