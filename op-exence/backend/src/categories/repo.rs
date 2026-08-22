use sqlx::PgPool;
use uuid::Uuid;

use crate::categories::model::{
    Category, CategoryListResponse, CreateCategoryRequest, ListCategoriesQuery,
    UpdateCategoryRequest,
};
use crate::error::AppError;
use crate::pagination::{normalize_limit, normalize_offset};

pub struct CategoryRepo;

impl CategoryRepo {
    pub async fn list(
        pool: &PgPool,
        query: ListCategoriesQuery,
    ) -> Result<CategoryListResponse, AppError> {
        let limit = normalize_limit(query.limit);
        let offset = normalize_offset(query.offset);

        let total: (i64,) = if let Some(ref group) = query.group {
            sqlx::query_as("SELECT COUNT(*) FROM categories WHERE category_group = $1")
                .bind(group)
                .fetch_one(pool)
                .await?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM categories")
                .fetch_one(pool)
                .await?
        };

        let items = if let Some(ref group) = query.group {
            sqlx::query_as::<_, Category>(
                r#"
                SELECT id, slug, name, category_group, description, is_system, created_at, updated_at
                FROM categories
                WHERE category_group = $1
                ORDER BY category_group, name
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(group)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Category>(
                r#"
                SELECT id, slug, name, category_group, description, is_system, created_at, updated_at
                FROM categories
                ORDER BY category_group, name
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        Ok(CategoryListResponse {
            items,
            limit,
            offset,
            total: total.0,
        })
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Category, AppError> {
        sqlx::query_as::<_, Category>(
            r#"
            SELECT id, slug, name, category_group, description, is_system, created_at, updated_at
            FROM categories
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
    }

    pub async fn create(pool: &PgPool, req: CreateCategoryRequest) -> Result<Category, AppError> {
        let id = Uuid::new_v4();
        let slug = req.slug.trim().to_string();
        let name = req.name.trim().to_string();
        let category_group = req.category_group.trim().to_string();
        let description = req
            .description
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty());

        sqlx::query_as::<_, Category>(
            r#"
            INSERT INTO categories (id, slug, name, category_group, description, is_system)
            VALUES ($1, $2, $3, $4, $5, FALSE)
            RETURNING id, slug, name, category_group, description, is_system, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(slug)
        .bind(name)
        .bind(category_group)
        .bind(description)
        .fetch_one(pool)
        .await
        .map_err(|err| {
            if let sqlx::Error::Database(db_err) = &err {
                if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                    return AppError::Conflict("slug already exists".to_string());
                }
            }
            AppError::from(err)
        })
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: UpdateCategoryRequest,
    ) -> Result<Category, AppError> {
        let existing = Self::get_by_id(pool, id).await?;

        if existing.is_system {
            return Err(AppError::Conflict(
                "system categories cannot be modified".to_string(),
            ));
        }

        let name = req
            .name
            .map(|n| n.trim().to_string())
            .unwrap_or(existing.name);
        let category_group = req
            .category_group
            .map(|g| g.trim().to_string())
            .unwrap_or(existing.category_group);
        let description = match req.description {
            Some(d) => {
                let trimmed = d.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }
            None => existing.description,
        };

        sqlx::query_as::<_, Category>(
            r#"
            UPDATE categories
            SET name = $2, category_group = $3, description = $4, updated_at = NOW()
            WHERE id = $1
            RETURNING id, slug, name, category_group, description, is_system, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(category_group)
        .bind(description)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        let existing = Self::get_by_id(pool, id).await?;

        if existing.is_system {
            return Err(AppError::Conflict(
                "system categories cannot be deleted".to_string(),
            ));
        }

        let expense_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM expenses WHERE category_id = $1")
                .bind(id)
                .fetch_one(pool)
                .await?;

        if expense_count.0 > 0 {
            return Err(AppError::Conflict(
                "category is referenced by expenses".to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }
}
