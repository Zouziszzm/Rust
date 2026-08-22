pub mod app;
pub mod categories;
pub mod config;
pub mod db;
pub mod error;
pub mod expenses;
pub mod shops;

pub mod pagination {
    pub const DEFAULT_LIMIT: i64 = 20;
    pub const MAX_LIMIT: i64 = 100;

    pub fn normalize_limit(limit: Option<i64>) -> i64 {
        limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
    }

    pub fn normalize_offset(offset: Option<i64>) -> i64 {
        offset.unwrap_or(0).max(0)
    }
}
