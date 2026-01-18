use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::json;

/// Custom JSON extractor that provides better error messages
///
/// This wrapper catches JSON deserialization errors (including validation errors
/// from custom deserializers) and formats them as standardized JSON error responses
/// instead of plain text.
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ValidationJsonRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(ValidatedJson(value)),
            Err(rejection) => Err(ValidationJsonRejection(rejection)),
        }
    }
}

/// Custom rejection type that formats JSON errors as standardized responses
pub struct ValidationJsonRejection(JsonRejection);

impl IntoResponse for ValidationJsonRejection {
    fn into_response(self) -> Response {
        // Extract the error message from the JsonRejection
        let error_message = self.0.to_string();
        
        // Determine status code and error details
        let (status, code, message) = if error_message.contains("Failed to deserialize") {
            // Parse validation error messages from custom deserializers
            let msg = if let Some(custom_msg) = error_message
                .split("Invalid stake amount:")
                .nth(1)
                .and_then(|s| s.split("at line").next())
                .map(|s| s.trim())
            {
                format!("Invalid stake amount: {}", custom_msg)
            } else if error_message.contains("missing field") {
                error_message
                    .split("missing field")
                    .nth(1)
                    .map(|s| format!("Missing required field{}", s.split("at line").next().unwrap_or("")))
                    .unwrap_or_else(|| "Missing required field".to_string())
            } else {
                "Invalid request body: failed to parse JSON".to_string()
            };
            
            (
                StatusCode::BAD_REQUEST,
                "VALIDATION_INVALID_INPUT",
                msg,
            )
        } else if error_message.contains("missing field") {
            let field = error_message
                .split("missing field `")
                .nth(1)
                .and_then(|s| s.split('`').next())
                .unwrap_or("unknown");
            (
                StatusCode::BAD_REQUEST,
                "VALIDATION_MISSING_FIELD",
                format!("Missing required field: {}", field),
            )
        } else {
            (
                StatusCode::BAD_REQUEST,
                "VALIDATION_INVALID_INPUT",
                "Invalid request body".to_string(),
            )
        };

        // Log the validation error
        tracing::warn!(
            error_code = code,
            error_message = %message,
            original_error = %error_message,
            "Request validation failed during JSON deserialization"
        );

        // Increment error metrics
        metrics::counter!("errors_total", "category" => "Validation", "code" => code).increment(1);

        // Return standardized JSON error response
        let body = Json(json!({
            "error": {
                "code": code,
                "message": message,
                "category": "Validation",
            }
        }));

        (status, body).into_response()
    }
}
