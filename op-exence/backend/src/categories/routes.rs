use axum::{
    Router,
    routing::get,
};

use crate::app::AppState;
use crate::categories::handlers::{
    create_category, delete_category, get_category, list_categories, update_category,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_categories).post(create_category))
        .route(
            "/{id}",
            get(get_category).patch(update_category).delete(delete_category),
        )
}
