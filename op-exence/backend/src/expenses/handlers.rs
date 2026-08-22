use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::AppError;
use crate::expenses::model::{
    CreateExpenseRequest, Expense, ExpenseListResponse, ExpenseSummaryResponse,
    ListExpensesQuery, MonthlySummaryResponse, SummaryQuery, UpdateExpenseRequest,
};
use crate::expenses::service::ExpenseService;

pub async fn list_expenses(
    State(state): State<AppState>,
    Query(query): Query<ListExpensesQuery>,
) -> Result<Json<ExpenseListResponse>, AppError> {
    let response = ExpenseService::list(&state.pool, query).await?;
    Ok(Json(response))
}

pub async fn get_expense(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Expense>, AppError> {
    let expense = ExpenseService::get(&state.pool, id).await?;
    Ok(Json(expense))
}

pub async fn create_expense(
    State(state): State<AppState>,
    Json(req): Json<CreateExpenseRequest>,
) -> Result<(StatusCode, Json<Expense>), AppError> {
    let expense = ExpenseService::create(&state.pool, req).await?;
    Ok((StatusCode::CREATED, Json(expense)))
}

pub async fn update_expense(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateExpenseRequest>,
) -> Result<Json<Expense>, AppError> {
    let expense = ExpenseService::update(&state.pool, id, req).await?;
    Ok(Json(expense))
}

pub async fn delete_expense(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    ExpenseService::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn expense_summary(
    State(state): State<AppState>,
    Query(query): Query<SummaryQuery>,
) -> Result<Json<ExpenseSummaryResponse>, AppError> {
    let summary = ExpenseService::summary(&state.pool, query).await?;
    Ok(Json(summary))
}

pub async fn monthly_summary(
    State(state): State<AppState>,
) -> Result<Json<MonthlySummaryResponse>, AppError> {
    let summary = ExpenseService::monthly_summary(&state.pool).await?;
    Ok(Json(summary))
}
