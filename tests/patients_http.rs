use axum::body::Body;
use axum::http::Request;
use axum_api::{app, AppState}; // crate name: axum-api -> axum_api
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt; // for oneshot

// can't collect here without http-body-util
async fn read_json(res: axum::response::Response) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[sqlx::test]
async fn create_patient_returns_patient(pool: PgPool) {
    let app = app(AppState { db: pool });

    let payload = json!({
        "first_name": "John",
        "last_name": "Doe",
        "birth_date": "1990-05-12",
        "email": "john@example.com"
    });

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/patients")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 201);
    let body = read_json(res).await;

    assert!(body["id"].as_str().unwrap().len() > 0);
    assert_eq!(body["first_name"], "John");
    assert_eq!(body["last_name"], "Doe");
    assert_eq!(body["birth_date"], "1990-05-12");
    assert_eq!(body["email"], "john@example.com");
    assert!(body["created_at"].as_str().unwrap().len() > 0);
    assert!(body["updated_at"].as_str().unwrap().len() > 0);
}

#[sqlx::test]
async fn search_patients_by_name_or_email(pool: PgPool) {
    let app = app(AppState { db: pool });

    // Insert 3 patients
    for (first, last, email) in [
        ("Alice", "Cooper", "alice@metal.com"),
        ("Bob", "Marley", "bob@reggae.com"),
        ("Charlie", "Parker", "charlie@jazz.com"),
    ] {
        let payload = json!({
            "first_name": first,
            "last_name": last,
            "birth_date": "1990-01-01",
            "email": email
        });

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/patients")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), 200);
    }

    // Search by last name fragment (ILIKE)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/patients?search=coop")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let list = read_json(res).await;
    let arr = list.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["first_name"], "Alice");

    // Search by email fragment
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/patients?search=jazz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let list = read_json(res).await;
    let arr = list.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["first_name"], "Charlie");

    // Search miss -> empty list
    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/patients?search=doesnotexist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let list = read_json(res).await;
    assert_eq!(list.as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn delete_patient_then_get_is_404(pool: PgPool) {
    let app = app(AppState { db: pool });

    // Create
    let payload = json!({
        "first_name": "John",
        "last_name": "Doe",
        "birth_date": "1990-05-12",
        "email": "john@example.com"
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/patients")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let created = read_json(res).await;
    let id = created["id"].as_str().unwrap();

    // Delete
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/patients/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 204);

    // Get should be 404
    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/patients/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[sqlx::test]
async fn delete_unknown_patient_is_404(pool: PgPool) {
    let app = app(AppState { db: pool });

    let id = uuid::Uuid::new_v4();
    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/patients/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}