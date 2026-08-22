use axum::{Router, routing::get};

use crate::app::AppState;
use crate::expenses::handlers::{
    create_expense, delete_expense, expense_summary, get_expense, list_expenses,
    monthly_summary, update_expense,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/summary", get(expense_summary))
        .route("/summary/monthly", get(monthly_summary))
        .route("/", get(list_expenses).post(create_expense))
        .route(
            "/{id}",
            get(get_expense).patch(update_expense).delete(delete_expense),
        )
}
