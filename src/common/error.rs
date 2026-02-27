use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,

    #[error("conflict")]
    Conflict,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("database error")]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: &'static str,
    message: Option<String>,
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Conflict => StatusCode::CONFLICT,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_key(&self) -> &'static str {
        match self {
            ApiError::NotFound => "not_found",
            ApiError::Conflict => "conflict",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::Db(_) => "internal_error",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        // Optional: keep messages only for safe errors
        let message = match &self {
            ApiError::BadRequest(msg) => Some(msg.clone()),
            ApiError::NotFound | ApiError::Conflict | ApiError::Db(_) => None,
        };

        let body = ErrorBody {
            error: self.error_key(),
            message,
        };

        (status, Json(body)).into_response()
    }
}

/// Maps SQLx errors into API errors (Postgres-focused but safe elsewhere).
pub fn map_sqlx_error(e: sqlx::Error) -> ApiError {
    // Postgres SQLSTATE:
    // 23505 = unique_violation
    if let sqlx::Error::Database(db_err) = &e {
        if db_err.code().as_deref() == Some("23505") {
            return ApiError::Conflict;
        }
    }
    ApiError::Db(e)
}


// Helper trait to convert SQLx results into API results with proper error mapping.
pub trait SqlxResultExt<T> {
    fn api_err(self) -> Result<T, ApiError>;
}

impl<T> SqlxResultExt<T> for Result<T, sqlx::Error> {
    fn api_err(self) -> Result<T, ApiError> {
        self.map_err(map_sqlx_error)
    }
}