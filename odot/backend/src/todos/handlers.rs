use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::AppError;
use crate::todos::model::{CreateTodoRequest, ListTodosQuery, Todo, UpdateTodoRequest};
use crate::todos::service::TodoService;

pub async fn list_todos(
    State(state): State<AppState>,
    Query(query): Query<ListTodosQuery>,
) -> Result<Json<crate::todos::model::TodoListResponse>, AppError> {
    let response = TodoService::list(&state.pool, query).await?;
    Ok(Json(response))
}

pub async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Todo>, AppError> {
    let todo = TodoService::get(&state.pool, id).await?;
    Ok(Json(todo))
}

pub async fn create_todo(
    State(state): State<AppState>,
    Json(req): Json<CreateTodoRequest>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let todo = TodoService::create(&state.pool, req).await?;
    Ok((StatusCode::CREATED, Json(todo)))
}

pub async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTodoRequest>,
) -> Result<Json<Todo>, AppError> {
    let todo = TodoService::update(&state.pool, id, req).await?;
    Ok(Json(todo))
}

pub async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    TodoService::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
