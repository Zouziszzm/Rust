use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_TITLE_LEN: usize = 255;
pub const MAX_DESCRIPTION_LEN: usize = 4096;
pub const DEFAULT_LIMIT: i64 = 20;
pub const MAX_LIMIT: i64 = 100;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListTodosQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TodoListResponse {
    pub items: Vec<Todo>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}

pub fn validate_title(title: &str) -> Result<(), String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err("title is required".to_string());
    }
    if trimmed.len() > MAX_TITLE_LEN {
        return Err(format!("title must be at most {MAX_TITLE_LEN} characters"));
    }
    Ok(())
}

pub fn validate_description(description: &Option<String>) -> Result<(), String> {
    if let Some(desc) = description {
        if desc.len() > MAX_DESCRIPTION_LEN {
            return Err(format!(
                "description must be at most {MAX_DESCRIPTION_LEN} characters"
            ));
        }
    }
    Ok(())
}

pub fn normalize_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

pub fn normalize_offset(offset: Option<i64>) -> i64 {
    offset.unwrap_or(0).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_title_rejects_empty() {
        assert!(validate_title("  ").is_err());
    }

    #[test]
    fn validate_title_rejects_too_long() {
        let title = "a".repeat(MAX_TITLE_LEN + 1);
        assert!(validate_title(&title).is_err());
    }

    #[test]
    fn validate_description_rejects_too_long() {
        let desc = Some("a".repeat(MAX_DESCRIPTION_LEN + 1));
        assert!(validate_description(&desc).is_err());
    }

    #[test]
    fn normalize_limit_clamps_values() {
        assert_eq!(normalize_limit(None), DEFAULT_LIMIT);
        assert_eq!(normalize_limit(Some(0)), 1);
        assert_eq!(normalize_limit(Some(500)), MAX_LIMIT);
    }

    #[test]
    fn normalize_offset_never_negative() {
        assert_eq!(normalize_offset(Some(-5)), 0);
    }
}
