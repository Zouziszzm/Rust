use axum::{
    Router,
    routing::get,
};

use crate::app::AppState;
use crate::todos::handlers::{
    create_todo, delete_todo, get_todo, list_todos, update_todo,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_todos).post(create_todo))
        .route("/{id}", get(get_todo).patch(update_todo).delete(delete_todo))
}
