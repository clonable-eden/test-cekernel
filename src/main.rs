use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::str::FromStr;
use test_cekernel::{app, setup_db};

#[tokio::main]
async fn main() {
    let options = SqliteConnectOptions::from_str("sqlite:todos.db")
        .expect("Invalid database URL")
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options)
        .await
        .expect("Failed to connect to database");

    setup_db(&pool).await;

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");
    println!("Listening on http://{addr}");
    axum::serve(listener, app(pool)).await.expect("Server failed");
}
