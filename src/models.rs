use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use sqlx::types::chrono;

// ==================== Wallet Models ====================

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
    pub fn as_str(&self) -> &'static str {
        match self {
            WalletType::Cash => "Cash",
            WalletType::BankAccount => "BankAccount",
            WalletType::CreditCard => "CreditCard",
            WalletType::Other => "Other",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Cash" => Some(WalletType::Cash),
            "BankAccount" => Some(WalletType::BankAccount),
            "CreditCard" => Some(WalletType::CreditCard),
            "Other" => Some(WalletType::Other),
            _ => None,
        }
    }

    pub fn is_credit_card(&self) -> bool {
        matches!(self, WalletType::CreditCard)
    }
}

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
    pub fn wallet_type_enum(&self) -> Option<WalletType> {
        WalletType::from_str(&self.wallet_type)
    }

    /// For credit cards: available balance = credit_limit - balance
    /// For others: available balance = balance
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

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub user_id: String,
    pub name: String,
    pub wallet_type: WalletType,
    #[serde(default)]
    pub balance: BigDecimal,
    pub credit_limit: Option<BigDecimal>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWalletRequest {
    pub name: Option<String>,
    pub balance: Option<BigDecimal>,
    pub credit_limit: Option<BigDecimal>,
}

// ==================== Transaction Models ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub wallet_id: String,  // Now required (not Option)
    pub amount: BigDecimal,  // Updated to BigDecimal for precision
    pub transaction_type: String, // "income" or "expense"
    pub category: String,
    pub description: Option<String>,  // Made optional to match schema
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub user_id: String,
    pub wallet_id: String,
    pub amount: BigDecimal,
    pub transaction_type: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTransactionRequest {
    pub wallet_id: Option<String>,
    pub amount: Option<BigDecimal>,
    pub category: Option<String>,
    pub description: Option<String>,
}

// ==================== Debt Models ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Debt {
    pub id: String,
    pub user_id: String,
    pub creditor_name: String,
    pub amount: BigDecimal,  // Updated to BigDecimal for precision
    pub interest_rate: BigDecimal,  // Updated to BigDecimal
    pub due_date: Option<DateTime<Utc>>,  // Made optional
    pub status: String, // "active", "paid", or "cancelled"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDebtRequest {
    pub user_id: String,
    pub creditor_name: String,
    pub amount: BigDecimal,  // Updated to BigDecimal
    pub interest_rate: Option<BigDecimal>,  // Made optional, defaults to 0
    pub due_date: Option<DateTime<Utc>>,  // Made optional
}

#[derive(Debug, Deserialize)]
pub struct UpdateDebtRequest {
    pub creditor_name: Option<String>,
    pub amount: Option<BigDecimal>,  // Updated to BigDecimal
    pub interest_rate: Option<BigDecimal>,  // Updated to BigDecimal
    pub due_date: Option<DateTime<Utc>>,
    pub status: Option<String>,
}

// ==================== API Response Models ====================

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}
