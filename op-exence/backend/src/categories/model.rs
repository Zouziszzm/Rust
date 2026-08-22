use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_NAME_LEN: usize = 100;
pub const MAX_SLUG_LEN: usize = 50;
pub const MAX_GROUP_LEN: usize = 50;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Category {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub category_group: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub slug: String,
    pub name: String,
    pub category_group: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub category_group: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListCategoriesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub group: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CategoryListResponse {
    pub items: Vec<Category>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}

pub fn validate_slug(slug: &str) -> Result<(), String> {
    let trimmed = slug.trim();
    if trimmed.is_empty() {
        return Err("slug is required".to_string());
    }
    if trimmed.len() > MAX_SLUG_LEN {
        return Err(format!("slug must be at most {MAX_SLUG_LEN} characters"));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err("slug must contain only lowercase letters, digits, and underscores".to_string());
    }
    Ok(())
}

pub fn validate_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("name is required".to_string());
    }
    if trimmed.len() > MAX_NAME_LEN {
        return Err(format!("name must be at most {MAX_NAME_LEN} characters"));
    }
    Ok(())
}

pub fn validate_group(group: &str) -> Result<(), String> {
    let trimmed = group.trim();
    if trimmed.is_empty() {
        return Err("category_group is required".to_string());
    }
    if trimmed.len() > MAX_GROUP_LEN {
        return Err(format!(
            "category_group must be at most {MAX_GROUP_LEN} characters"
        ));
    }
    Ok(())
}
