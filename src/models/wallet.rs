use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;

// ==================== WalletType Enum ====================

/// Enumeration of wallet types for organizing user finances
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WalletType {
    #[serde(rename = "Cash")]
    Cash,
    #[serde(rename = "BankAccount")]
    BankAccount,
    #[serde(rename = "CreditCard")]
    CreditCard,
    #[serde(rename = "Other")]
    Other,
}

impl WalletType {
    /// Convert enum variant to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            WalletType::Cash => "Cash",
            WalletType::BankAccount => "BankAccount",
            WalletType::CreditCard => "CreditCard",
            WalletType::Other => "Other",
        }
    }

    /// Parse string to WalletType enum
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Cash" => Some(WalletType::Cash),
            "BankAccount" => Some(WalletType::BankAccount),
            "CreditCard" => Some(WalletType::CreditCard),
            "Other" => Some(WalletType::Other),
            _ => None,
        }
    }

    /// Check if wallet is a credit card
    pub fn is_credit_card(&self) -> bool {
        matches!(self, WalletType::CreditCard)
    }
}

// ==================== Wallet Model ====================

/// Represents a user's wallet account
///
/// For CreditCard wallets:
/// - `balance` = current debt (0 = no debt, limit = fully used)
/// - `available_balance()` = credit_limit - balance
///
/// For other wallet types:
/// - `balance` = current balance
/// - `available_balance()` = balance
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Wallet {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub balance: BigDecimal,
    pub credit_limit: Option<BigDecimal>,
    pub wallet_type: String, // Stored as string from database
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Wallet {
    /// Get wallet type enum from string
    ///
    /// # Returns
    /// `Some(WalletType)` if the string is a valid wallet type, `None` otherwise
    pub fn wallet_type_enum(&self) -> Option<WalletType> {
        WalletType::from_str(&self.wallet_type)
    }

    /// Calculate available balance based on wallet type
    ///
    /// For credit cards: `available = credit_limit - balance`
    /// For others: `available = balance`
    pub fn available_balance(&self) -> BigDecimal {
        if let Some(limit) = &self.credit_limit {
            if self.wallet_type == "CreditCard" {
                // balance represents current debt, so available = limit - debt
                limit - &self.balance
            } else {
                self.balance.clone()
            }
        } else {
            self.balance.clone()
        }
    }
}

// ==================== Wallet Request Models ====================

/// Request to create a new wallet
#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub user_id: String,
    pub name: String,
    pub wallet_type: WalletType,
    #[serde(default)]
    pub balance: BigDecimal,
    pub credit_limit: Option<BigDecimal>,
}

/// Request to update an existing wallet
#[derive(Debug, Deserialize)]
pub struct UpdateWalletRequest {
    pub name: Option<String>,
    pub balance: Option<BigDecimal>,
    pub credit_limit: Option<BigDecimal>,
}
