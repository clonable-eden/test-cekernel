use axum::body::Body;
use axum::http::{Method, Request};
use axum::http::StatusCode;
use axum::Router;
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use tower::ServiceExt;

use test_cekernel::{app, setup_db, Todo};

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
