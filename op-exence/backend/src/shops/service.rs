use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::shops::model::{Shop, ShopListResponse};
use crate::shops::repo::ShopRepo;

pub struct ShopService;

impl ShopService {
    pub async fn list(pool: &PgPool) -> Result<ShopListResponse, AppError> {
        ShopRepo::list(pool).await
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Shop, AppError> {
        ShopRepo::get_by_id(pool, id).await
    }
}
