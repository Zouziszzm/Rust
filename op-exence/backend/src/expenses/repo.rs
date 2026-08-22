use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::expenses::model::{
    CategorySummary, Expense, ExpenseFields, ExpenseListResponse, ExpenseSummaryResponse,
    GroupSummary, ListExpensesQuery, MonthlySummaryItem, MonthlySummaryResponse, ShopSummary,
    SummaryQuery, TaxBreakdown,
};
use crate::pagination::{normalize_limit, normalize_offset};

const EXPENSE_SELECT: &str = r#"
    SELECT
        e.id, e.category_id, e.shop_id, e.amount_total,
        e.subtotal_10_percent, e.tax_amount_10_percent,
        e.subtotal_8_percent, e.tax_amount_8_percent,
        e.tax_exempt_amount, e.tax_amount_total,
        e.merchant_name, e.invoice_registration_number, e.is_qualified_invoice,
        e.receipt_number, e.title, e.notes, e.occurred_at,
        e.payment_method, e.is_recurring, e.is_refundable,
        e.created_at, e.updated_at,
        c.name AS category_name, c.category_group AS category_group,
        s.name AS shop_name
    FROM expenses e
    INNER JOIN categories c ON e.category_id = c.id
    LEFT JOIN shops s ON e.shop_id = s.id
"#;

pub struct ExpenseRepo;

impl ExpenseRepo {
    pub async fn list(
        pool: &PgPool,
        query: ListExpensesQuery,
    ) -> Result<ExpenseListResponse, AppError> {
        let limit = normalize_limit(query.limit);
        let offset = normalize_offset(query.offset);

        let mut conditions = Vec::new();
        let mut bind_idx = 1;

        if query.category_id.is_some() {
            conditions.push(format!("e.category_id = ${bind_idx}"));
            bind_idx += 1;
        }
        if query.group.is_some() {
            conditions.push(format!("c.category_group = ${bind_idx}"));
            bind_idx += 1;
        }
        if query.shop_id.is_some() {
            conditions.push(format!("e.shop_id = ${bind_idx}"));
            bind_idx += 1;
        }
        if query.payment_method.is_some() {
            conditions.push(format!("e.payment_method = ${bind_idx}"));
            bind_idx += 1;
        }
        if query.is_recurring.is_some() {
            conditions.push(format!("e.is_recurring = ${bind_idx}"));
            bind_idx += 1;
        }
        if query.from.is_some() {
            conditions.push(format!("e.occurred_at >= ${bind_idx}"));
            bind_idx += 1;
        }
        if query.to.is_some() {
            conditions.push(format!("e.occurred_at <= ${bind_idx}"));
            bind_idx += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_sql = format!(
            "SELECT COUNT(*) FROM expenses e INNER JOIN categories c ON e.category_id = c.id {where_clause}"
        );
        let list_sql = format!(
            "{EXPENSE_SELECT} {where_clause} ORDER BY e.occurred_at DESC LIMIT ${bind_idx} OFFSET ${}",
            bind_idx + 1
        );

        let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql);
        let mut list_query = sqlx::query_as::<_, Expense>(&list_sql);

        if let Some(category_id) = query.category_id {
            count_query = count_query.bind(category_id);
            list_query = list_query.bind(category_id);
        }
        if let Some(ref group) = query.group {
            count_query = count_query.bind(group);
            list_query = list_query.bind(group);
        }
        if let Some(shop_id) = query.shop_id {
            count_query = count_query.bind(shop_id);
            list_query = list_query.bind(shop_id);
        }
        if let Some(ref payment_method) = query.payment_method {
            count_query = count_query.bind(payment_method);
            list_query = list_query.bind(payment_method);
        }
        if let Some(is_recurring) = query.is_recurring {
            count_query = count_query.bind(is_recurring);
            list_query = list_query.bind(is_recurring);
        }
        if let Some(from) = query.from {
            count_query = count_query.bind(from);
            list_query = list_query.bind(from);
        }
        if let Some(to) = query.to {
            count_query = count_query.bind(to);
            list_query = list_query.bind(to);
        }

        list_query = list_query.bind(limit).bind(offset);

        let total = count_query.fetch_one(pool).await?;
        let items = list_query.fetch_all(pool).await?;

        Ok(ExpenseListResponse {
            items,
            limit,
            offset,
            total: total.0,
        })
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Expense, AppError> {
        let sql = format!("{EXPENSE_SELECT} WHERE e.id = $1");
        sqlx::query_as::<_, Expense>(&sql)
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn create(pool: &PgPool, fields: ExpenseFields) -> Result<Expense, AppError> {
        let id = Uuid::new_v4();
        let sql = format!(
            "{EXPENSE_SELECT} WHERE e.id = $1"
        );

        sqlx::query(
            r#"
            INSERT INTO expenses (
                id, category_id, shop_id, amount_total,
                subtotal_10_percent, tax_amount_10_percent,
                subtotal_8_percent, tax_amount_8_percent,
                tax_exempt_amount, tax_amount_total,
                merchant_name, invoice_registration_number, is_qualified_invoice,
                receipt_number, title, notes, occurred_at,
                payment_method, is_recurring, is_refundable
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            )
            "#,
        )
        .bind(id)
        .bind(fields.category_id)
        .bind(fields.shop_id)
        .bind(fields.amount_total)
        .bind(fields.subtotal_10_percent)
        .bind(fields.tax_amount_10_percent)
        .bind(fields.subtotal_8_percent)
        .bind(fields.tax_amount_8_percent)
        .bind(fields.tax_exempt_amount)
        .bind(fields.tax_amount_total)
        .bind(fields.merchant_name)
        .bind(fields.invoice_registration_number)
        .bind(fields.is_qualified_invoice)
        .bind(fields.receipt_number)
        .bind(fields.title)
        .bind(fields.notes)
        .bind(fields.occurred_at)
        .bind(fields.payment_method)
        .bind(fields.is_recurring)
        .bind(fields.is_refundable)
        .execute(pool)
        .await?;

        sqlx::query_as::<_, Expense>(&sql)
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(AppError::from)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        fields: ExpenseFields,
    ) -> Result<Expense, AppError> {
        let sql = format!("{EXPENSE_SELECT} WHERE e.id = $1");

        sqlx::query(
            r#"
            UPDATE expenses SET
                category_id = $2, shop_id = $3, amount_total = $4,
                subtotal_10_percent = $5, tax_amount_10_percent = $6,
                subtotal_8_percent = $7, tax_amount_8_percent = $8,
                tax_exempt_amount = $9, tax_amount_total = $10,
                merchant_name = $11, invoice_registration_number = $12,
                is_qualified_invoice = $13, receipt_number = $14,
                title = $15, notes = $16, occurred_at = $17,
                payment_method = $18, is_recurring = $19, is_refundable = $20,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(fields.category_id)
        .bind(fields.shop_id)
        .bind(fields.amount_total)
        .bind(fields.subtotal_10_percent)
        .bind(fields.tax_amount_10_percent)
        .bind(fields.subtotal_8_percent)
        .bind(fields.tax_amount_8_percent)
        .bind(fields.tax_exempt_amount)
        .bind(fields.tax_amount_total)
        .bind(fields.merchant_name)
        .bind(fields.invoice_registration_number)
        .bind(fields.is_qualified_invoice)
        .bind(fields.receipt_number)
        .bind(fields.title)
        .bind(fields.notes)
        .bind(fields.occurred_at)
        .bind(fields.payment_method)
        .bind(fields.is_recurring)
        .bind(fields.is_refundable)
        .execute(pool)
        .await?;

        sqlx::query_as::<_, Expense>(&sql)
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(AppError::from)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM expenses WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub async fn summary(
        pool: &PgPool,
        query: SummaryQuery,
    ) -> Result<ExpenseSummaryResponse, AppError> {
        let (date_filter, binds): (String, Vec<DateTime<Utc>>) = build_date_filter(&query);

        let total_sql = format!(
            "SELECT COALESCE(SUM(e.amount_total), 0)::bigint FROM expenses e INNER JOIN categories c ON e.category_id = c.id {date_filter}"
        );
        let mut total_query = sqlx::query_as::<_, (i64,)>(&total_sql);
        for b in &binds {
            total_query = total_query.bind(b);
        }
        let total = total_query.fetch_one(pool).await?.0;

        let group_sql = format!(
            r#"
            SELECT c.category_group AS group, COALESCE(SUM(e.amount_total), 0)::bigint AS total, COUNT(*)::bigint AS count
            FROM expenses e INNER JOIN categories c ON e.category_id = c.id
            {date_filter}
            {group_filter}
            GROUP BY c.category_group ORDER BY c.category_group
            "#,
            group_filter = if query.group.is_some() {
                if date_filter.is_empty() {
                    "WHERE c.category_group = $1"
                } else {
                    "AND c.category_group = $3"
                }
            } else {
                ""
            }
        );

        let mut group_query = sqlx::query_as::<_, GroupSummary>(&group_sql);
        for b in &binds {
            group_query = group_query.bind(b);
        }
        if let Some(ref group) = query.group {
            group_query = group_query.bind(group);
        }
        let by_group = group_query.fetch_all(pool).await?;

        let category_sql = format!(
            r#"
            SELECT c.id AS category_id, c.name AS category_name, c.category_group AS group,
                   COALESCE(SUM(e.amount_total), 0)::bigint AS total, COUNT(*)::bigint AS count
            FROM expenses e INNER JOIN categories c ON e.category_id = c.id
            {date_filter}
            {group_filter}
            GROUP BY c.id, c.name, c.category_group ORDER BY total DESC
            "#,
            group_filter = if query.group.is_some() {
                if date_filter.is_empty() {
                    "WHERE c.category_group = $1"
                } else {
                    "AND c.category_group = $3"
                }
            } else {
                ""
            }
        );

        let mut category_query = sqlx::query_as::<_, CategorySummary>(&category_sql);
        for b in &binds {
            category_query = category_query.bind(b);
        }
        if let Some(ref group) = query.group {
            category_query = category_query.bind(group);
        }
        let by_category = category_query.fetch_all(pool).await?;

        let shop_sql = format!(
            r#"
            SELECT s.id AS shop_id, s.name AS shop_name,
                   COALESCE(SUM(e.amount_total), 0)::bigint AS total, COUNT(*)::bigint AS count
            FROM expenses e INNER JOIN shops s ON e.shop_id = s.id
            INNER JOIN categories c ON e.category_id = c.id
            {date_filter}
            {group_filter}
            GROUP BY s.id, s.name ORDER BY total DESC
            "#,
            group_filter = if query.group.is_some() {
                if date_filter.is_empty() {
                    "WHERE c.category_group = $1"
                } else {
                    "AND c.category_group = $3"
                }
            } else {
                ""
            }
        );

        let mut shop_query = sqlx::query_as::<_, ShopSummary>(&shop_sql);
        for b in &binds {
            shop_query = shop_query.bind(b);
        }
        if let Some(ref group) = query.group {
            shop_query = shop_query.bind(group);
        }
        let by_shop = shop_query.fetch_all(pool).await?;

        let tax_sql = format!(
            r#"
            SELECT
                COALESCE(SUM(e.tax_amount_10_percent), 0)::bigint,
                COALESCE(SUM(e.tax_amount_8_percent), 0)::bigint,
                COALESCE(SUM(e.tax_amount_total), 0)::bigint
            FROM expenses e INNER JOIN categories c ON e.category_id = c.id
            {date_filter}
            {group_filter}
            "#,
            group_filter = if query.group.is_some() {
                if date_filter.is_empty() {
                    "WHERE c.category_group = $1"
                } else {
                    "AND c.category_group = $3"
                }
            } else {
                ""
            }
        );

        let mut tax_query = sqlx::query_as::<_, (i64, i64, i64)>(&tax_sql);
        for b in &binds {
            tax_query = tax_query.bind(b);
        }
        if let Some(ref group) = query.group {
            tax_query = tax_query.bind(group);
        }
        let (tax_10, tax_8, tax_total) = tax_query.fetch_one(pool).await?;

        let salary_deductions = if binds.is_empty() {
            sqlx::query_as::<_, (i64,)>(
                r#"
                SELECT COALESCE(SUM(e.amount_total), 0)::bigint
                FROM expenses e INNER JOIN categories c ON e.category_id = c.id
                WHERE c.category_group = 'insurance_tax' AND c.slug LIKE 'tax_%'
                "#,
            )
            .fetch_one(pool)
            .await?
            .0
        } else {
            let salary_sql = format!(
                r#"
                SELECT COALESCE(SUM(e.amount_total), 0)::bigint
                FROM expenses e INNER JOIN categories c ON e.category_id = c.id
                {date_filter}
                AND c.category_group = 'insurance_tax'
                AND c.slug LIKE 'tax_%'
                "#
            );
            let mut salary_query = sqlx::query_as::<_, (i64,)>(&salary_sql);
            for b in &binds {
                salary_query = salary_query.bind(b);
            }
            salary_query.fetch_one(pool).await?.0
        };

        Ok(ExpenseSummaryResponse {
            total,
            by_group,
            by_category,
            by_shop,
            tax_breakdown: TaxBreakdown {
                tax_10_percent: tax_10,
                tax_8_percent: tax_8,
                tax_total,
                salary_deductions_total: salary_deductions,
            },
        })
    }

    pub async fn monthly_summary(pool: &PgPool) -> Result<MonthlySummaryResponse, AppError> {
        let months = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT to_char(e.occurred_at, 'YYYY-MM') AS month, COALESCE(SUM(e.amount_total), 0)::bigint AS total
            FROM expenses e
            WHERE e.occurred_at >= NOW() - INTERVAL '12 months'
            GROUP BY to_char(e.occurred_at, 'YYYY-MM')
            ORDER BY month
            "#,
        )
        .fetch_all(pool)
        .await?;

        let mut items = Vec::new();
        for (month, total) in months {
            let by_group = sqlx::query_as::<_, GroupSummary>(
                r#"
                SELECT c.category_group AS group, COALESCE(SUM(e.amount_total), 0)::bigint AS total, COUNT(*)::bigint AS count
                FROM expenses e INNER JOIN categories c ON e.category_id = c.id
                WHERE to_char(e.occurred_at, 'YYYY-MM') = $1
                GROUP BY c.category_group ORDER BY c.category_group
                "#,
            )
            .bind(&month)
            .fetch_all(pool)
            .await?;

            items.push(MonthlySummaryItem {
                month,
                total,
                by_group,
            });
        }

        Ok(MonthlySummaryResponse { items })
    }
}

fn build_date_filter(query: &SummaryQuery) -> (String, Vec<DateTime<Utc>>) {
    let mut binds = Vec::new();
    let mut parts = Vec::new();

    if let Some(from) = query.from {
        parts.push(format!("e.occurred_at >= ${}", binds.len() + 1));
        binds.push(from);
    }
    if let Some(to) = query.to {
        parts.push(format!("e.occurred_at <= ${}", binds.len() + 1));
        binds.push(to);
    }

    let filter = if parts.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", parts.join(" AND "))
    };

    (filter, binds)
}

// sqlx FromRow for summary structs needs column aliases matching field names
// GroupSummary uses "group" - need to verify sqlx maps correctly with alias
