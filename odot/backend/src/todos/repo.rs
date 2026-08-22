use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::todos::model::{
    CreateTodoRequest, ListTodosQuery, Todo, TodoListResponse, UpdateTodoRequest,
    normalize_limit, normalize_offset,
};

pub struct TodoRepo;

impl TodoRepo {
    pub async fn list(pool: &PgPool, query: ListTodosQuery) -> Result<TodoListResponse, AppError> {
        let limit = normalize_limit(query.limit);
        let offset = normalize_offset(query.offset);

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM todos")
            .fetch_one(pool)
            .await?;

        let items = sqlx::query_as::<_, Todo>(
            r#"
            SELECT id, title, description, completed, created_at, updated_at
            FROM todos
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(TodoListResponse {
            items,
            limit,
            offset,
            total: total.0,
        })
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Todo, AppError> {
        let todo = sqlx::query_as::<_, Todo>(
            r#"
            SELECT id, title, description, completed, created_at, updated_at
            FROM todos
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

        Ok(todo)
    }

    pub async fn create(pool: &PgPool, req: CreateTodoRequest) -> Result<Todo, AppError> {
        let id = Uuid::new_v4();
        let title = req.title.trim().to_string();
        let description = req
            .description
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty());

        let todo = sqlx::query_as::<_, Todo>(
            r#"
            INSERT INTO todos (id, title, description)
            VALUES ($1, $2, $3)
            RETURNING id, title, description, completed, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(title)
        .bind(description)
        .fetch_one(pool)
        .await?;

        Ok(todo)
    }

    pub async fn update(pool: &PgPool, id: Uuid, req: UpdateTodoRequest) -> Result<Todo, AppError> {
        let existing = Self::get_by_id(pool, id).await?;

        let title = req
            .title
            .map(|t| t.trim().to_string())
            .unwrap_or(existing.title);
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
        let completed = req.completed.unwrap_or(existing.completed);

        let todo = sqlx::query_as::<_, Todo>(
            r#"
            UPDATE todos
            SET title = $2,
                description = $3,
                completed = $4,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, title, description, completed, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(title)
        .bind(description)
        .bind(completed)
        .fetch_one(pool)
        .await?;

        Ok(todo)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM todos WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }
}
