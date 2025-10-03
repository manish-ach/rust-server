mod auth;
mod model;
mod todos;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use rusqlite::Connection;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() {
    let conn = Connection::open("db.sqlite").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE,
            password TEXT
        )",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            task TEXT,
            completed INTEGER DEFAULT 0
        )",
        [],
    )
    .unwrap();

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        jwt_secret: "secret_key".to_string(),
    };

    let app = Router::new()
        .route("/auth/register", post(auth::verification::register))
        .route("/auth/login", post(auth::verification::login))
        .route("/todos", get(todos::get_todos))
        .route("/todos", post(todos::add_todo))
        .route("/todos/{id}", put(todos::update_todo))
        .route("/todos/{id}", delete(todos::delete_todo))
        .fallback_service(ServeDir::new("public"))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8001));
    println!("Server running at http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
