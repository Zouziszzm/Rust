use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_TITLE_LEN: usize = 200;
pub const MAX_MERCHANT_LEN: usize = 200;
pub const MAX_RECEIPT_LEN: usize = 100;
pub const MAX_INVOICE_LEN: usize = 14;

pub const SHOP_REQUIRED_GROUPS: &[&str] = &["groceries", "personal_care"];

pub const PAYMENT_METHODS: &[&str] = &[
    "cash",
    "credit_card",
    "debit_card",
    "paypay",
    "line_pay",
    "rakuten_pay",
    "suica_pasmo",
    "bank_transfer",
    "direct_debit",
    "other",
];

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Expense {
    pub id: Uuid,
    pub category_id: Uuid,
    pub shop_id: Option<Uuid>,
    pub amount_total: i64,
    pub subtotal_10_percent: i64,
    pub tax_amount_10_percent: i64,
    pub subtotal_8_percent: i64,
    pub tax_amount_8_percent: i64,
    pub tax_exempt_amount: i64,
    pub tax_amount_total: i64,
    pub merchant_name: Option<String>,
    pub invoice_registration_number: Option<String>,
    pub is_qualified_invoice: bool,
    pub receipt_number: Option<String>,
    pub title: String,
    pub notes: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub payment_method: String,
    pub is_recurring: bool,
    pub is_refundable: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub category_name: Option<String>,
    pub category_group: Option<String>,
    pub shop_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateExpenseRequest {
    pub category_id: Uuid,
    pub shop_id: Option<Uuid>,
    pub amount_total: i64,
    pub subtotal_10_percent: Option<i64>,
    pub tax_amount_10_percent: Option<i64>,
    pub subtotal_8_percent: Option<i64>,
    pub tax_amount_8_percent: Option<i64>,
    pub tax_exempt_amount: Option<i64>,
    pub tax_amount_total: Option<i64>,
    pub merchant_name: Option<String>,
    pub invoice_registration_number: Option<String>,
    pub is_qualified_invoice: Option<bool>,
    pub receipt_number: Option<String>,
    pub title: String,
    pub notes: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub payment_method: Option<String>,
    pub is_recurring: Option<bool>,
    pub is_refundable: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExpenseRequest {
    pub category_id: Option<Uuid>,
    pub shop_id: Option<Uuid>,
    pub amount_total: Option<i64>,
    pub subtotal_10_percent: Option<i64>,
    pub tax_amount_10_percent: Option<i64>,
    pub subtotal_8_percent: Option<i64>,
    pub tax_amount_8_percent: Option<i64>,
    pub tax_exempt_amount: Option<i64>,
    pub tax_amount_total: Option<i64>,
    pub merchant_name: Option<String>,
    pub invoice_registration_number: Option<String>,
    pub is_qualified_invoice: Option<bool>,
    pub receipt_number: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub payment_method: Option<String>,
    pub is_recurring: Option<bool>,
    pub is_refundable: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListExpensesQuery {
    pub category_id: Option<Uuid>,
    pub group: Option<String>,
    pub shop_id: Option<Uuid>,
    pub payment_method: Option<String>,
    pub is_recurring: Option<bool>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SummaryQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub group: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExpenseListResponse {
    pub items: Vec<Expense>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GroupSummary {
    #[sqlx(rename = "group")]
    pub group: String,
    pub total: i64,
    pub count: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CategorySummary {
    pub category_id: Uuid,
    pub category_name: String,
    #[sqlx(rename = "group")]
    pub group: String,
    pub total: i64,
    pub count: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ShopSummary {
    pub shop_id: Uuid,
    pub shop_name: String,
    pub total: i64,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct TaxBreakdown {
    pub tax_10_percent: i64,
    pub tax_8_percent: i64,
    pub tax_total: i64,
    pub salary_deductions_total: i64,
}

#[derive(Debug, Serialize)]
pub struct ExpenseSummaryResponse {
    pub total: i64,
    pub by_group: Vec<GroupSummary>,
    pub by_category: Vec<CategorySummary>,
    pub by_shop: Vec<ShopSummary>,
    pub tax_breakdown: TaxBreakdown,
}

#[derive(Debug, Serialize)]
pub struct MonthlySummaryItem {
    pub month: String,
    pub total: i64,
    pub by_group: Vec<GroupSummary>,
}

#[derive(Debug, Serialize)]
pub struct MonthlySummaryResponse {
    pub items: Vec<MonthlySummaryItem>,
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

pub fn validate_amount(amount: i64) -> Result<(), String> {
    if amount <= 0 {
        return Err("amount_total must be greater than 0".to_string());
    }
    Ok(())
}

pub fn validate_payment_method(method: &str) -> Result<(), String> {
    if PAYMENT_METHODS.contains(&method) {
        Ok(())
    } else {
        Err(format!(
            "payment_method must be one of: {}",
            PAYMENT_METHODS.join(", ")
        ))
    }
}

pub fn validate_invoice_number(num: &str) -> Result<(), String> {
    let trimmed = num.trim();
    if trimmed.len() != 14 || !trimmed.starts_with('T') {
        return Err("invoice_registration_number must be T followed by 13 digits".to_string());
    }
    if !trimmed[1..].chars().all(|c| c.is_ascii_digit()) {
        return Err("invoice_registration_number must be T followed by 13 digits".to_string());
    }
    Ok(())
}

pub fn validate_tax_totals(
    amount_total: i64,
    subtotal_10: i64,
    tax_10: i64,
    subtotal_8: i64,
    tax_8: i64,
    tax_exempt: i64,
) -> Result<(), String> {
    let has_tax_lines = subtotal_10 > 0 || tax_10 > 0 || subtotal_8 > 0 || tax_8 > 0 || tax_exempt > 0;
    if !has_tax_lines {
        return Ok(());
    }

    let computed = subtotal_10 + tax_10 + subtotal_8 + tax_8 + tax_exempt;
    let diff = (computed - amount_total).abs();
    if diff > 1 {
        return Err(format!(
            "tax breakdown ({computed}) does not match amount_total ({amount_total}); difference must be within 1 yen"
        ));
    }
    Ok(())
}

pub struct ExpenseFields {
    pub amount_total: i64,
    pub subtotal_10_percent: i64,
    pub tax_amount_10_percent: i64,
    pub subtotal_8_percent: i64,
    pub tax_amount_8_percent: i64,
    pub tax_exempt_amount: i64,
    pub tax_amount_total: i64,
    pub merchant_name: Option<String>,
    pub invoice_registration_number: Option<String>,
    pub is_qualified_invoice: bool,
    pub receipt_number: Option<String>,
    pub title: String,
    pub notes: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub payment_method: String,
    pub is_recurring: bool,
    pub is_refundable: bool,
    pub category_id: Uuid,
    pub shop_id: Option<Uuid>,
}

pub fn fields_from_create(req: CreateExpenseRequest) -> ExpenseFields {
    let subtotal_10 = req.subtotal_10_percent.unwrap_or(0);
    let tax_10 = req.tax_amount_10_percent.unwrap_or(0);
    let subtotal_8 = req.subtotal_8_percent.unwrap_or(0);
    let tax_8 = req.tax_amount_8_percent.unwrap_or(0);
    let tax_exempt = req.tax_exempt_amount.unwrap_or(0);
    let tax_total = req
        .tax_amount_total
        .unwrap_or_else(|| tax_10 + tax_8);

    ExpenseFields {
        amount_total: req.amount_total,
        subtotal_10_percent: subtotal_10,
        tax_amount_10_percent: tax_10,
        subtotal_8_percent: subtotal_8,
        tax_amount_8_percent: tax_8,
        tax_exempt_amount: tax_exempt,
        tax_amount_total: tax_total,
        merchant_name: clean_optional(req.merchant_name),
        invoice_registration_number: clean_optional(req.invoice_registration_number),
        is_qualified_invoice: req.is_qualified_invoice.unwrap_or(false),
        receipt_number: clean_optional(req.receipt_number),
        title: req.title.trim().to_string(),
        notes: clean_optional(req.notes),
        occurred_at: req.occurred_at,
        payment_method: req.payment_method.unwrap_or_else(|| "other".to_string()),
        is_recurring: req.is_recurring.unwrap_or(false),
        is_refundable: req.is_refundable.unwrap_or(false),
        category_id: req.category_id,
        shop_id: req.shop_id,
    }
}

pub fn fields_from_update(existing: &Expense, req: UpdateExpenseRequest) -> ExpenseFields {
    let subtotal_10 = req
        .subtotal_10_percent
        .unwrap_or(existing.subtotal_10_percent);
    let tax_10 = req.tax_amount_10_percent.unwrap_or(existing.tax_amount_10_percent);
    let subtotal_8 = req.subtotal_8_percent.unwrap_or(existing.subtotal_8_percent);
    let tax_8 = req.tax_amount_8_percent.unwrap_or(existing.tax_amount_8_percent);
    let tax_exempt = req.tax_exempt_amount.unwrap_or(existing.tax_exempt_amount);
    let tax_total = req
        .tax_amount_total
        .unwrap_or_else(|| tax_10 + tax_8);

    ExpenseFields {
        amount_total: req.amount_total.unwrap_or(existing.amount_total),
        subtotal_10_percent: subtotal_10,
        tax_amount_10_percent: tax_10,
        subtotal_8_percent: subtotal_8,
        tax_amount_8_percent: tax_8,
        tax_exempt_amount: tax_exempt,
        tax_amount_total: tax_total,
        merchant_name: match req.merchant_name {
            Some(v) => clean_optional(Some(v)),
            None => existing.merchant_name.clone(),
        },
        invoice_registration_number: match req.invoice_registration_number {
            Some(v) => clean_optional(Some(v)),
            None => existing.invoice_registration_number.clone(),
        },
        is_qualified_invoice: req
            .is_qualified_invoice
            .unwrap_or(existing.is_qualified_invoice),
        receipt_number: match req.receipt_number {
            Some(v) => clean_optional(Some(v)),
            None => existing.receipt_number.clone(),
        },
        title: req
            .title
            .map(|t| t.trim().to_string())
            .unwrap_or_else(|| existing.title.clone()),
        notes: match req.notes {
            Some(v) => clean_optional(Some(v)),
            None => existing.notes.clone(),
        },
        occurred_at: req.occurred_at.unwrap_or(existing.occurred_at),
        payment_method: req
            .payment_method
            .unwrap_or_else(|| existing.payment_method.clone()),
        is_recurring: req.is_recurring.unwrap_or(existing.is_recurring),
        is_refundable: req.is_refundable.unwrap_or(existing.is_refundable),
        category_id: req.category_id.unwrap_or(existing.category_id),
        shop_id: req.shop_id.or(existing.shop_id),
    }
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}
