use axum::{Router, routing::get};

use crate::app::AppState;
use crate::shops::handlers::{get_shop, list_shops};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shops))
        .route("/{id}", get(get_shop))
}
