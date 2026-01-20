use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared::errors::{ErrorCategory, ServiceError};
use serde_json::json;

/// AppError wraps the standardized ServiceError with service-specific conversions
///
/// This bridges the gap between external errors (Redis, etc.) and our
/// standardized error system. All errors are converted to ServiceError
/// internally and logged with structured context.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Service error: {0}")]
    Service(#[from] ServiceError),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Shared type validation error: {0}")]
    SharedValidation(#[from] shared::types::ValidationError),
}

impl AppError {
    /// Convert AppError to standardized ServiceError for consistent handling
    pub fn to_service_error(&self) -> ServiceError {
        match self {
            AppError::Service(e) => e.clone(),
            AppError::Redis(e) => ServiceError::redis_error(e),
            AppError::Internal(e) => ServiceError::internal(e.to_string()),
            AppError::SharedValidation(e) => {
                ServiceError::new(
                    ErrorCategory::Validation,
                    shared::errors::ErrorCode::VALIDATION_INVALID_AMOUNT,
                    e.to_string(),
                )
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let service_error = self.to_service_error();
        let status = StatusCode::from_u16(service_error.category.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Log error with structured fields based on severity
        match service_error.category {
            ErrorCategory::Internal | ErrorCategory::Network => {
                tracing::error!(
                    error_code = %service_error.code,
                    error_category = ?service_error.category,
                    error_message = %service_error.message,
                    error_context = ?service_error.context,
                    "Request failed with error"
                );
            }
            ErrorCategory::Validation | ErrorCategory::NotFound => {
                tracing::warn!(
                    error_code = %service_error.code,
                    error_category = ?service_error.category,
                    error_message = %service_error.message,
                    error_context = ?service_error.context,
                    "Request validation failed"
                );
            }
            ErrorCategory::Unauthorized => {
                tracing::warn!(
                    error_code = %service_error.code,
                    error_category = ?service_error.category,
                    error_message = %service_error.message,
                    "Unauthorized request"
                );
            }
            _ => {}
        }

        // Increment error metrics by category and code
        let category_str = format!("{:?}", service_error.category);
        metrics::counter!("errors_total", "category" => category_str, "code" => service_error.code.clone()).increment(1);

        // Return standardized JSON error response
        let body = Json(json!({
            "error": {
                "code": service_error.code,
                "message": service_error.message,
                "category": format!("{:?}", service_error.category),
            }
        }));

        (status, body).into_response()
    }
}

// Convenience constructors that return AppError::Service
impl AppError {
    pub fn not_found(message: impl Into<String>) -> Self {
        AppError::Service(ServiceError::new(
            ErrorCategory::NotFound,
            shared::errors::ErrorCode::NOT_FOUND_BET,
            message,
        ))
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        AppError::Service(ServiceError::new(
            ErrorCategory::Validation,
            shared::errors::ErrorCode::VALIDATION_INVALID_CHOICE,
            message,
        ))
    }

    pub fn insufficient_balance(required: i64, available: i64) -> Self {
        AppError::Service(ServiceError::insufficient_balance(required, available))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
