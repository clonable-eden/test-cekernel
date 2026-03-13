pub mod handlers;
pub mod models;

pub use handlers::AppState;
pub use models::{CreateTodo, Todo, UpdateTodo};

use axum::{routing::get, Router};
use sqlx::SqlitePool;
use std::sync::Arc;

use handlers::{create_todo, delete_todo, list_todos, update_todo};

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
            "/todos/{id}",
            axum::routing::patch(update_todo).delete(delete_todo),
        )
        .with_state(state)
}
