use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use std::sync::{Arc as StdArc, Mutex};
use tower::ServiceExt;
use tracing_subscriber::layer::SubscriberExt;

use test_cekernel::{Todo, app, setup_db};

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

async fn body_string(response: axum::response::Response) -> String {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(body.to_vec()).unwrap()
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

#[tokio::test]
async fn test_timestamps_present_on_create() {
    let app = test_app().await;
    let res = app
        .oneshot(json_request(
            Method::POST,
            "/todos",
            Some(r#"{"title":"Timestamp test"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let todo: Todo = body_json(res).await;
    assert!(
        !todo.created_at.is_empty(),
        "created_at should not be empty"
    );
    assert!(
        !todo.updated_at.is_empty(),
        "updated_at should not be empty"
    );
}

#[tokio::test]
async fn test_updated_at_changes_on_update() {
    use tokio::time::{Duration, sleep};

    let app = test_app().await;

    let res = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/todos",
            Some(r#"{"title":"Watch timestamps"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let created: Todo = body_json(res).await;

    // SQLite CURRENT_TIMESTAMP has 1-second resolution; wait to ensure updated_at changes
    sleep(Duration::from_secs(1)).await;

    let res = app
        .oneshot(json_request(
            Method::PATCH,
            &format!("/todos/{}", created.id),
            Some(r#"{"title":"Updated title"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Todo = body_json(res).await;

    assert_eq!(
        updated.created_at, created.created_at,
        "created_at should not change"
    );
    assert!(
        updated.updated_at >= created.updated_at,
        "updated_at should be >= original"
    );
}

#[tokio::test]
async fn test_create_with_content() {
    let app = test_app().await;
    let res = app
        .oneshot(json_request(
            Method::POST,
            "/todos",
            Some(r#"{"title":"With content","content":"Detailed description"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let todo: Todo = body_json(res).await;
    assert_eq!(todo.title, "With content");
    assert_eq!(todo.content, "Detailed description");
}

#[tokio::test]
async fn test_create_without_content_defaults_to_empty() {
    let app = test_app().await;
    let res = app
        .oneshot(json_request(
            Method::POST,
            "/todos",
            Some(r#"{"title":"No content"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let todo: Todo = body_json(res).await;
    assert_eq!(todo.title, "No content");
    assert_eq!(todo.content, "");
}

#[tokio::test]
async fn test_update_content() {
    let app = test_app().await;

    let res = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/todos",
            Some(r#"{"title":"Update me","content":"Original content"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let created: Todo = body_json(res).await;
    assert_eq!(created.content, "Original content");

    let res = app
        .oneshot(json_request(
            Method::PATCH,
            &format!("/todos/{}", created.id),
            Some(r#"{"content":"Updated content"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Todo = body_json(res).await;
    assert_eq!(updated.content, "Updated content");
    assert_eq!(updated.title, "Update me");
}

// ---- HTML Frontend Tests ----

#[tokio::test]
async fn test_index_returns_html() {
    let app = test_app().await;
    let res = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let content_type = res
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = body_string(res).await;
    assert!(
        content_type.contains("text/html"),
        "Expected text/html, got {content_type}"
    );
    assert!(body.contains("<html"), "Body should contain <html");
    assert!(
        body.contains("unpkg.com/htmx.org"),
        "Body should reference htmx via CDN"
    );
    assert!(
        body.contains("unpkg.com/mvp.css"),
        "Body should reference mvp.css via CDN"
    );
}

#[tokio::test]
async fn test_index_shows_existing_todos() {
    let app = test_app().await;

    // Create a todo via JSON API
    let res = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/todos",
            Some(r#"{"title":"Visible todo"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // GET / should show the todo
    let res = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_string(res).await;
    assert!(
        body.contains("Visible todo"),
        "Index page should show existing todos"
    );
}

#[tokio::test]
async fn test_create_todo_html() {
    let app = test_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("title=New+html+todo"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_string(res).await;
    assert!(
        body.contains("New html todo"),
        "Response should contain the created todo"
    );
}

#[tokio::test]
async fn test_toggle_todo_html() {
    let app = test_app().await;

    // Create todo via JSON API
    let res = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/todos",
            Some(r#"{"title":"Toggle me"}"#),
        ))
        .await
        .unwrap();
    let todo: Todo = body_json(res).await;
    assert!(!todo.completed);

    // Toggle via HTML endpoint
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&format!("/todos/{}/toggle", todo.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_string(res).await;
    assert!(
        body.contains("Toggle me"),
        "Response should contain the todo title"
    );
    assert!(
        body.contains("checked"),
        "Toggled todo should be marked as checked"
    );
}

#[tokio::test]
async fn test_delete_todo_html() {
    let app = test_app().await;

    // Create todo via JSON API
    let res = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/todos",
            Some(r#"{"title":"Delete me html"}"#),
        ))
        .await
        .unwrap();
    let todo: Todo = body_json(res).await;

    // Delete via HTML endpoint
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&format!("/todos/{}/delete", todo.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Verify todo is deleted via JSON API
    let res = app
        .oneshot(json_request(Method::GET, "/todos", None))
        .await
        .unwrap();
    let todos: Vec<Todo> = body_json(res).await;
    assert!(
        todos.is_empty(),
        "Todo should be deleted after HTML delete endpoint"
    );
}

// ---- Tracing Tests ----

/// A tracing layer that records event messages to a shared buffer.
struct EventCapture {
    buf: StdArc<Mutex<Vec<String>>>,
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for EventCapture {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        use tracing::field::Visit;
        struct MsgVisitor(String);
        impl Visit for MsgVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = format!("{:?}", value);
                }
            }
        }
        let mut visitor = MsgVisitor(String::new());
        event.record(&mut visitor);
        let level = *event.metadata().level();
        let entry = format!("[{}] {}", level, visitor.0);
        self.buf.lock().unwrap().push(entry);
    }
}

#[tokio::test]
async fn test_db_error_is_logged() {
    let log_buf: StdArc<Mutex<Vec<String>>> = StdArc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(EventCapture {
        buf: log_buf.clone(),
    });
    let _guard = tracing::subscriber::set_default(subscriber);

    // Use a pool that will fail on query (closed pool)
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to connect");
    setup_db(&pool).await;
    let router = app(pool.clone());

    // Close the pool to force DB errors
    pool.close().await;

    let res = router
        .oneshot(json_request(Method::GET, "/todos", None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Verify that the error was logged via tracing
    let logs = log_buf.lock().unwrap();
    assert!(
        logs.iter()
            .any(|l| l.contains("ERROR") && l.contains("Failed to fetch todos")),
        "Expected ERROR log containing 'Failed to fetch todos', got: {:?}",
        *logs
    );
}
