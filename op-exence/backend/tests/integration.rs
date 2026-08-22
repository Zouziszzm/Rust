use axum::body::Body;
use http_body_util::BodyExt;
use op_exence::app::{build_router, AppState};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let pool = sqlx::PgPool::connect_lazy("postgres://op_exence:op_exence@localhost:5432/op_exence")
        .expect("pool");
    let state = AppState { pool };
    let app = build_router(state, "http://localhost:8081");

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
