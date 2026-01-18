//! Bet status serialization and deserialization
//!
//! Converts between BetStatus enum and Redis string representations.

use crate::domain::BetStatus;

/// Convert BetStatus to Redis string
pub fn status_to_string(status: &BetStatus) -> String {
    match status {
        BetStatus::Pending => "pending",
        BetStatus::Batched => "batched",
        BetStatus::SubmittedToSolana => "submitted_to_solana",
        BetStatus::ConfirmedOnSolana => "confirmed_on_solana",
        BetStatus::Completed => "completed",
        BetStatus::FailedRetryable => "failed_retryable",
        BetStatus::FailedManualReview => "failed_manual_review",
    }
    .to_string()
}

/// Parse BetStatus from Redis string
pub fn status_from_string(s: &str) -> Option<BetStatus> {
    match s {
        "pending" => Some(BetStatus::Pending),
        "batched" => Some(BetStatus::Batched),
        "submitted_to_solana" => Some(BetStatus::SubmittedToSolana),
        "confirmed_on_solana" => Some(BetStatus::ConfirmedOnSolana),
        "completed" => Some(BetStatus::Completed),
        "failed_retryable" => Some(BetStatus::FailedRetryable),
        "failed_manual_review" => Some(BetStatus::FailedManualReview),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_round_trip() {
        let statuses = vec![
            BetStatus::Pending,
            BetStatus::Batched,
            BetStatus::SubmittedToSolana,
            BetStatus::ConfirmedOnSolana,
            BetStatus::Completed,
            BetStatus::FailedRetryable,
            BetStatus::FailedManualReview,
        ];

        for status in statuses {
            let serialized = status_to_string(&status);
            let deserialized = status_from_string(&serialized);
            assert_eq!(deserialized, Some(status));
        }
    }

    #[test]
    fn test_invalid_status_string() {
        assert_eq!(status_from_string("invalid"), None);
        assert_eq!(status_from_string(""), None);
    }
}
