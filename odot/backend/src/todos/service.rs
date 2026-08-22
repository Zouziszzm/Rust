use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::todos::model::{
    CreateTodoRequest, ListTodosQuery, Todo, TodoListResponse, UpdateTodoRequest,
    validate_description, validate_title,
};
use crate::todos::repo::TodoRepo;

pub struct TodoService;

impl TodoService {
    pub async fn list(pool: &PgPool, query: ListTodosQuery) -> Result<TodoListResponse, AppError> {
        TodoRepo::list(pool, query).await
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Todo, AppError> {
        TodoRepo::get_by_id(pool, id).await
    }

    pub async fn create(pool: &PgPool, req: CreateTodoRequest) -> Result<Todo, AppError> {
        validate_title(&req.title).map_err(AppError::Validation)?;
        validate_description(&req.description).map_err(AppError::Validation)?;
        TodoRepo::create(pool, req).await
    }

    pub async fn update(pool: &PgPool, id: Uuid, req: UpdateTodoRequest) -> Result<Todo, AppError> {
        if let Some(ref title) = req.title {
            validate_title(title).map_err(AppError::Validation)?;
        }
        if let Some(ref description) = req.description {
            validate_description(&Some(description.clone())).map_err(AppError::Validation)?;
        }
        TodoRepo::update(pool, id, req).await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        TodoRepo::delete(pool, id).await
    }
}
