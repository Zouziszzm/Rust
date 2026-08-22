use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::shops::model::{Shop, ShopListResponse};

pub struct ShopRepo;

impl ShopRepo {
    pub async fn list(pool: &PgPool) -> Result<ShopListResponse, AppError> {
        let items = sqlx::query_as::<_, Shop>(
            r#"
            SELECT id, slug, name, shop_type, is_system, created_at
            FROM shops
            ORDER BY name
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(ShopListResponse { items })
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Shop, AppError> {
        sqlx::query_as::<_, Shop>(
            r#"
            SELECT id, slug, name, shop_type, is_system, created_at
            FROM shops
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
    }
}
