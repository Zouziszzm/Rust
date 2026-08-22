use sqlx::PgPool;
use uuid::Uuid;

use crate::categories::model::{
    Category, CategoryListResponse, CreateCategoryRequest, ListCategoriesQuery,
    UpdateCategoryRequest, validate_group, validate_name, validate_slug,
};
use crate::categories::repo::CategoryRepo;
use crate::error::AppError;

pub struct CategoryService;

impl CategoryService {
    pub async fn list(
        pool: &PgPool,
        query: ListCategoriesQuery,
    ) -> Result<CategoryListResponse, AppError> {
        CategoryRepo::list(pool, query).await
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Category, AppError> {
        CategoryRepo::get_by_id(pool, id).await
    }

    pub async fn create(pool: &PgPool, req: CreateCategoryRequest) -> Result<Category, AppError> {
        validate_slug(&req.slug).map_err(AppError::Validation)?;
        validate_name(&req.name).map_err(AppError::Validation)?;
        validate_group(&req.category_group).map_err(AppError::Validation)?;
        CategoryRepo::create(pool, req).await
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: UpdateCategoryRequest,
    ) -> Result<Category, AppError> {
        if let Some(ref name) = req.name {
            validate_name(name).map_err(AppError::Validation)?;
        }
        if let Some(ref group) = req.category_group {
            validate_group(group).map_err(AppError::Validation)?;
        }
        CategoryRepo::update(pool, id, req).await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        CategoryRepo::delete(pool, id).await
    }
}
