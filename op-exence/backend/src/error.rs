use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),

    #[error("resource not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal server error")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    code: &'static str,
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::NotFound => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn message(&self) -> String {
        match self {
            AppError::Validation(msg) => msg.clone(),
            AppError::NotFound => "resource not found".to_string(),
            AppError::Conflict(msg) => msg.clone(),
            AppError::Internal(_) => "internal server error".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ErrorBody {
            error: self.message(),
            code: self.code(),
        };
        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound,
            other => {
                tracing::error!(error = %other, "database error");
                AppError::Internal(other.to_string())
            }
        }
    }
}
