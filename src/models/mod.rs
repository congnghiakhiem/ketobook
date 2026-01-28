// ==================== Module Exports ====================

/// Wallet module - User wallet accounts and types
pub mod wallet;
pub use wallet::{Wallet, WalletType, CreateWalletRequest, UpdateWalletRequest};

/// Transaction module - Financial transactions on wallets
pub mod transaction;
pub use transaction::{Transaction, CreateTransactionRequest, UpdateTransactionRequest};

/// Debt module - Debt and obligation tracking
pub mod debt;
pub use debt::{Debt, CreateDebtRequest, UpdateDebtRequest};

// ==================== Common API Response Model ====================

use serde::Serialize;

/// Generic API response wrapper
///
/// All API endpoints return responses wrapped in this structure,
/// with either data (on success) or error (on failure).
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response with data
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}
