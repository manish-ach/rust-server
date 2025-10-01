use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use bcrypt::hash;
use jsonwebtoken::{self, EncodingKey, Header, encode};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
// use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
    jwt_secret: String,
}

#[derive(Deserialize)]
struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
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
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .fallback_service(ServeDir::new("public"))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8001));
    println!("Server running at http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let hashed = hash(&payload.password, 8).unwrap();

    let db = state.db.lock().await;
    let result = db.execute(
        "INSERT INTO users (username, password) VALUES (?, ?)",
        params![payload.username, hashed],
    );
    if let Err(_) = result {
        return Err(StatusCode::CONFLICT);
    }

    let user_id = db.last_insert_rowid();

    db.execute(
        "INSERT INTO todos (user_id, task) VALUES (?, ?)",
        params![user_id, "Add a ToDo!"],
    )
    .unwrap();

    let claims = serde_json::json!({"id": user_id});
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .unwrap();

    Ok(Json(AuthResponse { token }))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let db = state.db.lock().await;

    let user_result = db.query_row(
        "SELECT id, password FROM users WHERE username = ?",
        params![&payload.username],
        |row| {
            let id: i64 = row.get(0)?;
            let password: String = row.get(1)?;
            Ok((id, password))
        },
    );

    let (user_id, hashed_password) = match user_result {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    let password_valid = bcrypt::verify(&payload.password, &hashed_password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !password_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let claims = serde_json::json!({"id": user_id});
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse { token }))
}
