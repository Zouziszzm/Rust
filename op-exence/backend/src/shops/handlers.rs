use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::AppError;
use crate::shops::model::{Shop, ShopListResponse};
use crate::shops::service::ShopService;

pub async fn list_shops(State(state): State<AppState>) -> Result<Json<ShopListResponse>, AppError> {
    let response = ShopService::list(&state.pool).await?;
    Ok(Json(response))
}

pub async fn get_shop(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Shop>, AppError> {
    let shop = ShopService::get(&state.pool, id).await?;
    Ok(Json(shop))
}
