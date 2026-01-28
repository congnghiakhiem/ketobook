use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;

// ==================== Debt Model ====================

/// Represents a debt (loan, credit, obligation)
///
/// Debts track financial obligations with optional links to wallets.
/// Unlike transactions which are wallet-specific, debts are user-level and may
/// be associated with multiple wallets or none at all.
///
/// Debt is preserved even if associated wallet is deleted (FK uses ON DELETE SET NULL).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Debt {
    pub id: String,
    pub user_id: String,
    pub wallet_id: Option<String>,        // Optional FK to wallets (SET NULL on delete)
    pub creditor_name: String,            // Name of creditor (bank, person, company)
    pub amount: BigDecimal,               // Principal debt amount
    pub interest_rate: BigDecimal,        // Annual interest rate as percentage
    pub due_date: Option<DateTime<Utc>>,  // Optional payment due date
    pub status: String,                   // "active", "paid", or "cancelled"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== Debt Request Models ====================

/// Request to create a new debt
#[derive(Debug, Deserialize)]
pub struct CreateDebtRequest {
    pub user_id: String,
    pub wallet_id: Option<String>,
    pub creditor_name: String,
    pub amount: BigDecimal,
    pub interest_rate: Option<BigDecimal>,
    pub due_date: Option<DateTime<Utc>>,
}

/// Request to update an existing debt
#[derive(Debug, Deserialize)]
pub struct UpdateDebtRequest {
    pub creditor_name: Option<String>,
    pub amount: Option<BigDecimal>,
    pub interest_rate: Option<BigDecimal>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: Option<String>,
}
