use axum::{Json, extract::State, http::StatusCode};
use bcrypt::{hash, verify};
use jsonwebtoken::{EncodingKey, Header, encode};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::model::Claims;

#[derive(Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let hashed = hash(&payload.password, 8).unwrap();
    let db = state.db.lock().await;

    let result = db.execute(
        "INSERT INTO users (username, password) VALUES (?, ?)",
        params![payload.username, hashed],
    );

    if result.is_err() {
        return Err(StatusCode::CONFLICT);
    }

    let user_id = db.last_insert_rowid();
    db.execute(
        "INSERT INTO todos (user_id, task) VALUES (?, ?)",
        params![user_id, "Add a ToDo!"],
    )
    .unwrap();

    let claims = Claims {
        id: user_id,
        exp: 2000000000,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .unwrap();

    Ok(Json(AuthResponse { token }))
}

pub async fn login(
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

    let password_valid = verify(&payload.password, &hashed_password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !password_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let claims = Claims {
        id: user_id,
        exp: 2000000000,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse { token }))
}
