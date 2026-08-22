#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    use odot::app::{AppState, build_router};
    use odot::config::Config;
    use odot::db;

    async fn setup_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://odot:odot@localhost:5432/odot".to_string());

        let pool = db::create_pool(&Config {
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_url,
            db_pool_max: 5,
            cors_origin: "http://localhost:3000".to_string(),
            reset_data: false,
        })
        .await
        .expect("failed to connect to test database");

        db::run_migrations(&pool)
            .await
            .expect("failed to run migrations");

        sqlx::query("TRUNCATE TABLE todos")
            .execute(&pool)
            .await
            .expect("failed to truncate todos");

        pool
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let pool = setup_pool().await;
        let app = build_router(AppState { pool }, "http://localhost:3000");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn todo_crud_flow() {
        let pool = setup_pool().await;
        let app = build_router(AppState { pool }, "http://localhost:3000");

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/todos")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"title":"Write tests","description":"integration"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::CREATED);

        let body = create_response.into_body().collect().await.unwrap().to_bytes();
        let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = created["id"].as_str().unwrap();
        assert_eq!(created["title"], "Write tests");

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/todos/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);

        let patch_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/todos/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"completed":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(patch_response.status(), StatusCode::OK);

        let delete_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/todos/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        let missing_response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/todos/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(missing_response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn create_rejects_empty_title() {
        let pool = setup_pool().await;
        let app = build_router(AppState { pool }, "http://localhost:3000");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/todos")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"   "}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(error["code"], "VALIDATION_ERROR");
    }

    #[tokio::test]
    async fn get_unknown_todo_returns_not_found() {
        let pool = setup_pool().await;
        let app = build_router(AppState { pool }, "http://localhost:3000");
        let id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/todos/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
