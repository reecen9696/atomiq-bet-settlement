/// Shared error types for atomik-wallet services
///
/// Design Philosophy:
/// - Standardized error codes for consistent error handling across services
/// - Categorized by error domain (Validation, Network, Contract, Internal)
/// - Implements both Display and thiserror::Error for compatibility
/// - Includes context fields for debugging (error_code, message, context)
///
/// Usage:
/// - Backend/Processor services wrap their specific errors in ServiceError
/// - Error codes follow pattern: <CATEGORY>_<SPECIFIC>_<DETAIL>
/// - Context field used for additional debugging information
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error categories that map to HTTP status codes and logging severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCategory {
    /// Validation errors (400 Bad Request)
    /// Client provided invalid input
    Validation,

    /// Network/RPC errors (502/503 Bad Gateway/Service Unavailable)
    /// External service is unavailable or timing out
    Network,

    /// Smart contract errors (400/500 depending on error)
    /// Solana transaction or program execution failed
    Contract,

    /// Internal service errors (500 Internal Server Error)
    /// Unexpected failures, database issues, programming errors
    Internal,

    /// Resource not found (404 Not Found)
    NotFound,

    /// Authorization/Authentication errors (401/403)
    Unauthorized,
}

impl ErrorCategory {
    /// Map error category to HTTP status code
    pub fn status_code(&self) -> u16 {
        match self {
            ErrorCategory::Validation => 400,
            ErrorCategory::Network => 503,
            ErrorCategory::Contract => 400,
            ErrorCategory::Internal => 500,
            ErrorCategory::NotFound => 404,
            ErrorCategory::Unauthorized => 401,
        }
    }

    /// Map error category to log level
    pub fn log_level(&self) -> &'static str {
        match self {
            ErrorCategory::Validation => "warn",
            ErrorCategory::Network => "error",
            ErrorCategory::Contract => "warn",
            ErrorCategory::Internal => "error",
            ErrorCategory::NotFound => "info",
            ErrorCategory::Unauthorized => "warn",
        }
    }
}

/// Standard error codes used across all services
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorCode(pub &'static str);

impl ErrorCode {
    // Validation errors
    pub const VALIDATION_INVALID_BET_ID: ErrorCode = ErrorCode("VALIDATION_INVALID_BET_ID");
    pub const VALIDATION_INVALID_AMOUNT: ErrorCode = ErrorCode("VALIDATION_INVALID_AMOUNT");
    pub const VALIDATION_INVALID_WALLET: ErrorCode = ErrorCode("VALIDATION_INVALID_WALLET");
    pub const VALIDATION_INVALID_CHOICE: ErrorCode = ErrorCode("VALIDATION_INVALID_CHOICE");
    pub const VALIDATION_INSUFFICIENT_BALANCE: ErrorCode =
        ErrorCode("VALIDATION_INSUFFICIENT_BALANCE");
    pub const VALIDATION_ALLOWANCE_EXPIRED: ErrorCode = ErrorCode("VALIDATION_ALLOWANCE_EXPIRED");

    // Network errors
    pub const NETWORK_RPC_UNAVAILABLE: ErrorCode = ErrorCode("NETWORK_RPC_UNAVAILABLE");
    pub const NETWORK_RPC_TIMEOUT: ErrorCode = ErrorCode("NETWORK_RPC_TIMEOUT");
    pub const NETWORK_REDIS_CONNECTION: ErrorCode = ErrorCode("NETWORK_REDIS_CONNECTION");
    pub const NETWORK_DATABASE_CONNECTION: ErrorCode = ErrorCode("NETWORK_DATABASE_CONNECTION");
    pub const NETWORK_BACKEND_UNAVAILABLE: ErrorCode = ErrorCode("NETWORK_BACKEND_UNAVAILABLE");

    // Smart contract errors
    pub const CONTRACT_EXECUTION_FAILED: ErrorCode = ErrorCode("CONTRACT_EXECUTION_FAILED");
    pub const CONTRACT_INSUFFICIENT_RENT: ErrorCode = ErrorCode("CONTRACT_INSUFFICIENT_RENT");
    pub const CONTRACT_INVALID_PDA: ErrorCode = ErrorCode("CONTRACT_INVALID_PDA");
    pub const CONTRACT_UNAUTHORIZED_SIGNER: ErrorCode = ErrorCode("CONTRACT_UNAUTHORIZED_SIGNER");
    pub const CONTRACT_ACCOUNT_NOT_FOUND: ErrorCode = ErrorCode("CONTRACT_ACCOUNT_NOT_FOUND");

    // Internal errors
    pub const INTERNAL_UNEXPECTED: ErrorCode = ErrorCode("INTERNAL_UNEXPECTED");
    pub const INTERNAL_SERIALIZATION: ErrorCode = ErrorCode("INTERNAL_SERIALIZATION");
    pub const INTERNAL_DESERIALIZATION: ErrorCode = ErrorCode("INTERNAL_DESERIALIZATION");
    pub const INTERNAL_DATABASE_QUERY: ErrorCode = ErrorCode("INTERNAL_DATABASE_QUERY");
    pub const INTERNAL_CONFIGURATION: ErrorCode = ErrorCode("INTERNAL_CONFIGURATION");

    // Resource errors
    pub const NOT_FOUND_BET: ErrorCode = ErrorCode("NOT_FOUND_BET");
    pub const NOT_FOUND_BATCH: ErrorCode = ErrorCode("NOT_FOUND_BATCH");
    pub const NOT_FOUND_VAULT: ErrorCode = ErrorCode("NOT_FOUND_VAULT");
    pub const NOT_FOUND_ALLOWANCE: ErrorCode = ErrorCode("NOT_FOUND_ALLOWANCE");

    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Standardized error structure used across all services
///
/// This provides consistent error reporting with:
/// - Structured error codes for programmatic handling
/// - Human-readable messages
/// - Optional context for debugging
/// - Category-based classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceError {
    /// Error category (determines status code and log level)
    pub category: ErrorCategory,
    
    /// Structured error code
    pub code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Optional additional context (e.g., field names, IDs, stack traces)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl ServiceError {
    /// Create a new ServiceError
    pub fn new(category: ErrorCategory, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            category,
            code: code.as_str().to_string(),
            message: message.into(),
            context: None,
        }
    }

    /// Add context to an error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    // Validation error constructors
    pub fn invalid_bet_id(bet_id: impl fmt::Display) -> Self {
        Self::new(
            ErrorCategory::Validation,
            ErrorCode::VALIDATION_INVALID_BET_ID,
            format!("Invalid bet ID: {}", bet_id),
        )
    }

    pub fn invalid_amount(amount: i64, reason: impl Into<String>) -> Self {
        Self::new(
            ErrorCategory::Validation,
            ErrorCode::VALIDATION_INVALID_AMOUNT,
            format!("Invalid amount: {}", amount),
        )
        .with_context(reason)
    }

    pub fn invalid_wallet(wallet: impl fmt::Display) -> Self {
        Self::new(
            ErrorCategory::Validation,
            ErrorCode::VALIDATION_INVALID_WALLET,
            format!("Invalid wallet address: {}", wallet),
        )
    }

    pub fn insufficient_balance(required: i64, available: i64) -> Self {
        Self::new(
            ErrorCategory::Validation,
            ErrorCode::VALIDATION_INSUFFICIENT_BALANCE,
            "Insufficient balance",
        )
        .with_context(format!("required: {}, available: {}", required, available))
    }

    // Network error constructors
    pub fn rpc_unavailable(endpoint: impl Into<String>) -> Self {
        Self::new(
            ErrorCategory::Network,
            ErrorCode::NETWORK_RPC_UNAVAILABLE,
            "Solana RPC endpoint unavailable",
        )
        .with_context(endpoint)
    }

    pub fn redis_error(error: impl fmt::Display) -> Self {
        Self::new(
            ErrorCategory::Network,
            ErrorCode::NETWORK_REDIS_CONNECTION,
            "Redis connection error",
        )
        .with_context(error.to_string())
    }

    pub fn database_error(error: impl fmt::Display) -> Self {
        Self::new(
            ErrorCategory::Network,
            ErrorCode::NETWORK_DATABASE_CONNECTION,
            "Database connection error",
        )
        .with_context(error.to_string())
    }

    // Contract error constructors
    pub fn contract_execution_failed(tx_signature: impl Into<String>, error: impl fmt::Display) -> Self {
        Self::new(
            ErrorCategory::Contract,
            ErrorCode::CONTRACT_EXECUTION_FAILED,
            "Smart contract execution failed",
        )
        .with_context(format!("tx: {}, error: {}", tx_signature.into(), error))
    }

    pub fn invalid_pda(expected: impl fmt::Display, actual: impl fmt::Display) -> Self {
        Self::new(
            ErrorCategory::Contract,
            ErrorCode::CONTRACT_INVALID_PDA,
            "Invalid PDA derivation",
        )
        .with_context(format!("expected: {}, actual: {}", expected, actual))
    }

    // Resource not found constructors
    pub fn bet_not_found(bet_id: impl fmt::Display) -> Self {
        Self::new(
            ErrorCategory::NotFound,
            ErrorCode::NOT_FOUND_BET,
            format!("Bet not found: {}", bet_id),
        )
    }

    pub fn batch_not_found(batch_id: impl fmt::Display) -> Self {
        Self::new(
            ErrorCategory::NotFound,
            ErrorCode::NOT_FOUND_BATCH,
            format!("Batch not found: {}", batch_id),
        )
    }

    // Internal error constructors
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCategory::Internal,
            ErrorCode::INTERNAL_UNEXPECTED,
            message,
        )
    }

    pub fn serialization_error(error: impl fmt::Display) -> Self {
        Self::new(
            ErrorCategory::Internal,
            ErrorCode::INTERNAL_SERIALIZATION,
            "Serialization error",
        )
        .with_context(error.to_string())
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(context) = &self.context {
            write!(f, "[{}] {}: {}", self.code, self.message, context)
        } else {
            write!(f, "[{}] {}", self.code, self.message)
        }
    }
}

impl std::error::Error for ServiceError {}

// Convenience type alias
pub type Result<T> = std::result::Result<T, ServiceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_status_codes() {
        assert_eq!(ErrorCategory::Validation.status_code(), 400);
        assert_eq!(ErrorCategory::Network.status_code(), 503);
        assert_eq!(ErrorCategory::NotFound.status_code(), 404);
        assert_eq!(ErrorCategory::Internal.status_code(), 500);
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(
            ErrorCode::VALIDATION_INVALID_BET_ID.to_string(),
            "VALIDATION_INVALID_BET_ID"
        );
    }

    #[test]
    fn test_service_error_creation() {
        let error = ServiceError::invalid_bet_id("test-123");
        assert_eq!(error.category, ErrorCategory::Validation);
        assert_eq!(error.code, "VALIDATION_INVALID_BET_ID");
        assert!(error.message.contains("test-123"));
    }

    #[test]
    fn test_service_error_with_context() {
        let error = ServiceError::invalid_amount(100, "below minimum")
            .with_context("min: 1000");
        assert!(error.context.is_some());
        assert!(error.to_string().contains("min: 1000"));
    }

    #[test]
    fn test_error_serialization() {
        let error = ServiceError::bet_not_found("abc-123");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("NOT_FOUND_BET"));
        assert!(json.contains("abc-123"));
    }
}
