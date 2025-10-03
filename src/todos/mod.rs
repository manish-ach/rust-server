use crate::{AppState, auth::extractor::AuthUser, model::Todo};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use rusqlite::{Result, params};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateTodo {
    pub task: String,
}

#[derive(Deserialize)]
pub struct UpdateTodo {
    pub completed: bool,
}

pub async fn get_todos(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<Vec<Todo>>, StatusCode> {
    let db = state.db.lock().await;

    let mut stmt = db
        .prepare("SELECT id, user_id, task, completed FROM todos WHERE user_id = ?")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let row = stmt
        .query_map([user_id], |row| {
            Ok(Todo {
                id: row.get(0)?,
                user_id: row.get(1)?,
                task: row.get(2)?,
                completed: row.get::<_, i32>(3)? != 0,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let todos: Vec<Todo> = row.filter_map(Result::ok).collect();
    Ok(Json(todos))
}

pub async fn add_todo(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(payload): Json<CreateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let db = state.db.lock().await;
    db.execute(
        "INSERT INTO todos (user_id, task) VALUES (?, ?)",
        params![user_id, payload.task],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let id = db.last_insert_rowid();

    Ok(Json(Todo {
        id,
        user_id,
        task: payload.task,
        completed: false,
    }))
}

pub async fn update_todo(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<String>, StatusCode> {
    let db = state.db.lock().await;

    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM todos WHERE id = ? AND user_id = ?",
            params![id, user_id],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if count == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    db.execute(
        "UPDATE todos SET completed = ? WHERE id = ?",
        params![payload.completed as i32, id],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("Todo Updated".to_string()))
}

pub async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<String>, StatusCode> {
    let db = state.db.lock().await;

    db.execute(
        "DELETE FROM todos WHERE id = ? AND user_id = ?",
        params![id, user_id],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("Todo deleted".to_string()))
}
