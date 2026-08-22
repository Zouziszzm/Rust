use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::app::AppState;
use crate::categories::model::{
    Category, CategoryListResponse, CreateCategoryRequest, ListCategoriesQuery,
    UpdateCategoryRequest,
};
use crate::categories::service::CategoryService;
use crate::error::AppError;

pub async fn list_categories(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<CategoryListResponse>, AppError> {
    let response = CategoryService::list(&state.pool, query).await?;
    Ok(Json(response))
}

pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Category>, AppError> {
    let category = CategoryService::get(&state.pool, id).await?;
    Ok(Json(category))
}

pub async fn create_category(
    State(state): State<AppState>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<Category>), AppError> {
    let category = CategoryService::create(&state.pool, req).await?;
    Ok((StatusCode::CREATED, Json(category)))
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCategoryRequest>,
) -> Result<Json<Category>, AppError> {
    let category = CategoryService::update(&state.pool, id, req).await?;
    Ok(Json(category))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    CategoryService::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
