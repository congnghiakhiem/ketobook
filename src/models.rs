use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ==================== Transaction Models ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub amount: f64,
    pub transaction_type: String, // "income" or "expense"
    pub category: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub user_id: String,
    pub amount: f64,
    pub transaction_type: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTransactionRequest {
    pub amount: Option<f64>,
    pub category: Option<String>,
    pub description: Option<String>,
}

// ==================== Debt Models ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Debt {
    pub id: String,
    pub user_id: String,
    pub creditor_name: String,
    pub amount: f64,
    pub interest_rate: f64,
    pub due_date: DateTime<Utc>,
    pub status: String, // "active" or "paid"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDebtRequest {
    pub user_id: String,
    pub creditor_name: String,
    pub amount: f64,
    pub interest_rate: f64,
    pub due_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDebtRequest {
    pub creditor_name: Option<String>,
    pub amount: Option<f64>,
    pub interest_rate: Option<f64>,
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
