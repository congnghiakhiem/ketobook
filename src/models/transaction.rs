use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use uuid::Uuid;

// ==================== Transaction Model ====================

/// Represents a financial transaction on a wallet
///
/// Transactions record monetary movements (income or expense) against a wallet.
/// All amounts are stored as positive values with the type determining the operation:
/// - "income": adds to wallet balance
/// - "expense": subtracts from wallet balance
///
/// Transactions are linked to a wallet via `wallet_id` foreign key,
/// and cascade-delete when the wallet is deleted.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: String,
    pub wallet_id: Uuid,                  // Required FK to wallets
    pub amount: BigDecimal,               // Always positive; type determines operation
    pub transaction_type: String,         // "income" or "expense"
    pub category: String,                 // Transaction category (e.g., groceries, salary)
    pub description: Option<String>,      // Optional details
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== Transaction Request Models ====================

/// Request to create a new transaction
#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub user_id: String,
    pub wallet_id: Uuid,
    pub amount: BigDecimal,
    pub transaction_type: String,         // "income" or "expense"
    pub category: String,
    pub description: String,
}

/// Request to update an existing transaction
#[derive(Debug, Deserialize)]
pub struct UpdateTransactionRequest {
    pub wallet_id: Option<Uuid>,
    pub amount: Option<BigDecimal>,
    pub category: Option<String>,
    pub description: Option<String>,
}
