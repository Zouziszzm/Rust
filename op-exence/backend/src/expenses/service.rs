use sqlx::PgPool;
use uuid::Uuid;

use crate::categories::repo::CategoryRepo;
use crate::error::AppError;
use crate::expenses::model::{
    CreateExpenseRequest, Expense, ExpenseFields, ExpenseListResponse, ExpenseSummaryResponse,
    ListExpensesQuery, MonthlySummaryResponse, SHOP_REQUIRED_GROUPS, SummaryQuery,
    UpdateExpenseRequest, fields_from_create, fields_from_update, validate_amount,
    validate_invoice_number, validate_payment_method, validate_tax_totals, validate_title,
};
use crate::expenses::repo::ExpenseRepo;

pub struct ExpenseService;

impl ExpenseService {
    pub async fn list(pool: &PgPool, query: ListExpensesQuery) -> Result<ExpenseListResponse, AppError> {
        ExpenseRepo::list(pool, query).await
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Expense, AppError> {
        ExpenseRepo::get_by_id(pool, id).await
    }

    pub async fn create(pool: &PgPool, req: CreateExpenseRequest) -> Result<Expense, AppError> {
        let fields = fields_from_create(req);
        validate_expense_fields(pool, &fields).await?;
        ExpenseRepo::create(pool, fields).await
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: UpdateExpenseRequest,
    ) -> Result<Expense, AppError> {
        let existing = ExpenseRepo::get_by_id(pool, id).await?;
        let fields = fields_from_update(&existing, req);
        validate_expense_fields(pool, &fields).await?;
        ExpenseRepo::update(pool, id, fields).await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        ExpenseRepo::delete(pool, id).await
    }

    pub async fn summary(pool: &PgPool, query: SummaryQuery) -> Result<ExpenseSummaryResponse, AppError> {
        ExpenseRepo::summary(pool, query).await
    }

    pub async fn monthly_summary(pool: &PgPool) -> Result<MonthlySummaryResponse, AppError> {
        ExpenseRepo::monthly_summary(pool).await
    }
}

async fn validate_expense_fields(pool: &PgPool, fields: &ExpenseFields) -> Result<(), AppError> {
    validate_title(&fields.title).map_err(AppError::Validation)?;
    validate_amount(fields.amount_total).map_err(AppError::Validation)?;
    validate_payment_method(&fields.payment_method).map_err(AppError::Validation)?;
    validate_tax_totals(
        fields.amount_total,
        fields.subtotal_10_percent,
        fields.tax_amount_10_percent,
        fields.subtotal_8_percent,
        fields.tax_amount_8_percent,
        fields.tax_exempt_amount,
    )
    .map_err(AppError::Validation)?;

    if let Some(ref invoice) = fields.invoice_registration_number {
        validate_invoice_number(invoice).map_err(AppError::Validation)?;
    }

    let category = CategoryRepo::get_by_id(pool, fields.category_id).await?;

    if SHOP_REQUIRED_GROUPS.contains(&category.category_group.as_str()) && fields.shop_id.is_none() {
        return Err(AppError::Validation(
            "shop_id is required for groceries and personal care categories".to_string(),
        ));
    }

    Ok(())
}
