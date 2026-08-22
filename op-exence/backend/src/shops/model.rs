use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Shop {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub shop_type: String,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ShopListResponse {
    pub items: Vec<Shop>,
}
