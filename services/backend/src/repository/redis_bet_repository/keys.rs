//! Redis key generation functions
//!
//! Centralizes all Redis key patterns used for bet storage and indexing.

use uuid::Uuid;

/// Redis key prefix for bets
const BET_KEY_PREFIX: &str = "bet:";

/// Redis key prefix for user-bet index
const USER_INDEX_PREFIX: &str = "bets:user:";

/// Redis key for claimable bets sorted set
const CLAIMABLE_INDEX: &str = "bets:claimable";

/// Redis key for processing bets sorted set
const PROCESSING_INDEX: &str = "bets:processing";

/// Generate Redis key for a bet
pub fn bet_key(bet_id: Uuid) -> String {
    format!("{}{}", BET_KEY_PREFIX, bet_id)
}

/// Generate Redis key for user's bet index
pub fn user_index_key(user_wallet: &str) -> String {
    format!("{}{}", USER_INDEX_PREFIX, user_wallet)
}

/// Get Redis key for claimable bets index
pub fn claimable_index_key() -> &'static str {
    CLAIMABLE_INDEX
}

/// Get Redis key for processing bets index
pub fn processing_index_key() -> &'static str {
    PROCESSING_INDEX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bet_key_format() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(bet_key(id), "bet:550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_user_index_key_format() {
        assert_eq!(
            user_index_key("EXAMPLEpubkey123"),
            "bets:user:EXAMPLEpubkey123"
        );
    }

    #[test]
    fn test_index_keys_are_constants() {
        assert_eq!(claimable_index_key(), "bets:claimable");
        assert_eq!(processing_index_key(), "bets:processing");
    }
}
