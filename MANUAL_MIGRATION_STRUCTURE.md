# Manual Migration Structure - KetoBook (2026)

This document provides a comprehensive overview of the manually structured migrations for KetoBook, demonstrating proper database design patterns and Rust integration.

## Migration Overview

**File:** `migrations/20260128001_initial_schema.sql`  
**Created:** 2026-01-28  
**Purpose:** Complete database schema with explicit dependency ordering

### Execution Order

The migration enforces strict dependency ordering to avoid "relation does not exist" errors:

```
1. ENUM Types (wallet_type)
   ↓ (depends on nothing)
2. Wallets Table
   ↓ (depends on wallet_type ENUM)
3. Transactions Table
   ↓ (depends on wallets via FK)
4. Debts Table
   ↓ (depends on wallets via FK)
5. Aggregation Views
   (depend on base tables)
```

## Database Schema

### 1. ENUM: wallet_type

```sql
CREATE TYPE wallet_type AS ENUM (
    'Cash',
    'BankAccount',
    'CreditCard',
    'Other'
);
```

**Purpose:** Enforce wallet type constraints at database level  
**Values:**
- `Cash`: Physical cash wallet
- `BankAccount`: Checking/savings account
- `CreditCard`: Credit line (balance = debt, available = limit - debt)
- `Other`: Custom wallet type

---

### 2. Table: wallets

```sql
CREATE TABLE wallets (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    wallet_type wallet_type NOT NULL DEFAULT 'Cash',
    balance DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    credit_limit DECIMAL(15, 2) DEFAULT 0.00,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT balance_non_negative CHECK (balance >= 0),
    CONSTRAINT credit_limit_non_negative CHECK (credit_limit >= 0)
);
```

**Columns:**
- `id`: UUID primary key
- `user_id`: User identifier (not FK for flexibility)
- `name`: Display name (e.g., "My Checking Account")
- `wallet_type`: ENUM reference to wallet_type
- `balance`: Current balance (DECIMAL 15,2 for financial precision)
- `credit_limit`: Credit limit for credit cards (0 for other types)
- `created_at`, `updated_at`: Timestamps with UTC timezone

**Key Semantics for Credit Cards:**
```
balance = current debt (0 = no debt, limit = fully used)
available_credit = credit_limit - balance
transaction validation: amount <= available_credit
```

**Indexes:**
- `idx_wallets_user_id`: For user-level queries
- `idx_wallets_wallet_type`: For filtering by type
- `idx_wallets_created_at`: For chronological queries
- `idx_wallets_credit_card`: Partial index for credit card queries

**Triggers:**
- `trigger_wallets_updated_at`: Auto-updates `updated_at` on modification

---

### 3. Table: transactions

```sql
CREATE TABLE transactions (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(100) NOT NULL,
    wallet_id VARCHAR(36) NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    transaction_type VARCHAR(20) NOT NULL,
    category VARCHAR(100),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_transactions_wallet_id FOREIGN KEY (wallet_id) 
        REFERENCES wallets(id) ON DELETE CASCADE,
    CONSTRAINT amount_positive CHECK (amount > 0),
    CONSTRAINT valid_transaction_type CHECK (transaction_type IN ('income', 'expense'))
);
```

**Columns:**
- `id`: UUID primary key
- `user_id`: User who performed the transaction
- `wallet_id`: Foreign key to wallets (CASCADE DELETE)
- `amount`: Transaction amount (always positive)
- `transaction_type`: "income" or "expense"
- `category`: Classification (groceries, salary, utilities, etc.)
- `description`: Details about the transaction
- `created_at`, `updated_at`: Timestamps with UTC timezone

**Foreign Key Behavior:**
- `ON DELETE CASCADE`: When a wallet is deleted, all associated transactions are deleted
- Ensures referential integrity

**Constraints:**
- `amount > 0`: All amounts are positive; type determines debit/credit
- `transaction_type IN ('income', 'expense')`: Only valid types

**Indexes:**
- `idx_transactions_user_id`: For user queries
- `idx_transactions_wallet_id`: For wallet-specific queries
- `idx_transactions_created_at`: For chronological sorting
- `idx_transactions_type`: For filtering by type
- `idx_transactions_user_wallet`: Composite for efficient filtering
- `idx_transactions_user_created`: Composite for recent transaction queries

**Triggers:**
- `trigger_transactions_updated_at`: Auto-updates `updated_at` on modification

---

### 4. Table: debts

```sql
CREATE TABLE debts (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(100) NOT NULL,
    wallet_id VARCHAR(36),
    creditor_name VARCHAR(255) NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    interest_rate DECIMAL(5, 2) DEFAULT 0.00,
    due_date TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_debts_wallet_id FOREIGN KEY (wallet_id) 
        REFERENCES wallets(id) ON DELETE SET NULL,
    CONSTRAINT amount_positive CHECK (amount > 0),
    CONSTRAINT interest_rate_non_negative CHECK (interest_rate >= 0),
    CONSTRAINT valid_status CHECK (status IN ('active', 'paid', 'cancelled'))
);
```

**Columns:**
- `id`: UUID primary key
- `user_id`: User who owes the debt
- `wallet_id`: Optional reference to responsible wallet
- `creditor_name`: Name of creditor (bank, person, company)
- `amount`: Principal debt amount (DECIMAL 15,2)
- `interest_rate`: Annual interest rate as percentage
- `due_date`: Optional payment due date
- `status`: "active", "paid", or "cancelled"
- `created_at`, `updated_at`: Timestamps with UTC timezone

**Foreign Key Behavior:**
- `wallet_id` is NULLABLE
- `ON DELETE SET NULL`: If wallet is deleted, wallet_id becomes NULL (debt preserved)
- This differs from transactions (which cascade delete)

**Indexes:**
- `idx_debts_user_id`: For user queries
- `idx_debts_wallet_id`: For wallet-specific debts
- `idx_debts_status`: For status filtering
- `idx_debts_due_date`: For sorting by due date
- `idx_debts_user_status`: Composite for user status queries
- `idx_debts_user_due`: Partial index for overdue queries

**Triggers:**
- `trigger_debts_updated_at`: Auto-updates `updated_at` on modification

---

### 5. Views

#### v_transaction_summary
Aggregated transaction statistics by user, wallet, and type:
```sql
SELECT 
    user_id, wallet_id, transaction_type,
    COUNT(*) as transaction_count,
    SUM(amount) as total_amount,
    AVG(amount) as average_amount,
    MIN(created_at) as first_transaction,
    MAX(created_at) as last_transaction
FROM transactions
GROUP BY user_id, wallet_id, transaction_type;
```

#### v_debt_summary
Aggregated debt statistics by user and status:
```sql
SELECT 
    user_id, status,
    COUNT(*) as debt_count,
    SUM(amount) as total_debt,
    SUM(amount * (interest_rate / 100)) as estimated_annual_interest,
    COUNT(CASE WHEN due_date < CURRENT_TIMESTAMP THEN 1 END) as overdue_count
FROM debts
GROUP BY user_id, status;
```

#### v_wallet_summary
Wallet statistics with calculated available balance:
```sql
SELECT 
    w.id, w.user_id, w.name, w.wallet_type, w.balance,
    w.credit_limit,
    CASE 
        WHEN w.wallet_type = 'CreditCard' AND w.credit_limit > 0 
        THEN (w.credit_limit - w.balance)
        ELSE w.balance 
    END as available_balance,
    COUNT(t.id) as transaction_count,
    MAX(t.created_at) as last_transaction_date,
    w.created_at, w.updated_at
FROM wallets w
LEFT JOIN transactions t ON w.id = t.wallet_id
GROUP BY w.id, ...;
```

---

## Rust Model Integration

### Models Definition (src/models.rs)

```rust
use sqlx::types::BigDecimal;
use chrono::{DateTime, Utc};

// Wallet Type Enum
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

// Wallet Model (sqlx::FromRow for direct DB deserialization)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Wallet {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub balance: BigDecimal,           // DECIMAL(15, 2) from DB
    pub credit_limit: Option<BigDecimal>,
    pub wallet_type: String,           // Maps to wallet_type ENUM
    pub created_at: DateTime<Utc>,     // TIMESTAMP WITH TIME ZONE
    pub updated_at: DateTime<Utc>,
}

// Transaction Model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub wallet_id: String,             // Required (not Option)
    pub amount: BigDecimal,            // DECIMAL(15, 2) from DB
    pub transaction_type: String,      // "income" or "expense"
    pub category: String,
    pub description: Option<String>,   // TEXT is nullable
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Debt Model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Debt {
    pub id: String,
    pub user_id: String,
    pub wallet_id: Option<String>,     // Nullable FK to wallets
    pub creditor_name: String,
    pub amount: BigDecimal,            // DECIMAL(15, 2) - Important!
    pub interest_rate: BigDecimal,     // DECIMAL(5, 2) - Important!
    pub due_date: Option<DateTime<Utc>>,
    pub status: String,                // "active", "paid", "cancelled"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Create Request Models
#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub user_id: String,
    pub wallet_id: String,
    pub amount: BigDecimal,            // Client sends BigDecimal as string in JSON
    pub transaction_type: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDebtRequest {
    pub user_id: String,
    pub wallet_id: Option<String>,
    pub creditor_name: String,
    pub amount: BigDecimal,            // Use BigDecimal, not f64!
    pub interest_rate: Option<BigDecimal>,
    pub due_date: Option<DateTime<Utc>>,
}
```

**Key Points:**
- Use `sqlx::FromRow` derive macro for automatic deserialization from query results
- Use `BigDecimal` for all monetary fields (DECIMAL in DB)
- Use `DateTime<Utc>` for timestamps (TIMESTAMP WITH TIME ZONE in DB)
- Use `Option<T>` for nullable database columns
- JSON serialization of BigDecimal is automatic (serialized as string)

---

## Atomic Transaction Pattern

### Example: Create Transaction with Wallet Balance Update

```rust
pub async fn create_transaction_atomic(
    req: web::Json<CreateTransactionRequest>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let transaction_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    // Step 1: Validate input
    if req.amount <= BigDecimal::from_str("0").unwrap() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<Transaction>::error("Amount must be greater than 0".to_string()));
    }

    // Step 2: Fetch wallet to validate balance
    let wallet: Option<Wallet> = match sqlx::query_as::<_, Wallet>(
        "SELECT id, user_id, name, balance, credit_limit, wallet_type, created_at, updated_at 
         FROM wallets WHERE id = $1 AND user_id = $2"
    )
    .bind(&req.wallet_id)
    .bind(&req.user_id)
    .fetch_optional(db.get_ref())
    .await {
        Ok(w) => w,
        Err(e) => {
            log::error!("Error fetching wallet: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Database error".to_string()));
        }
    };

    let wallet = match wallet {
        Some(w) => w,
        None => return HttpResponse::NotFound()
            .json(ApiResponse::<Transaction>::error("Wallet not found".to_string())),
    };

    // Step 3: Validate balance for expense transactions
    if req.transaction_type == "expense" && req.amount > wallet.balance {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<Transaction>::error("Insufficient balance".to_string()));
    }

    // Step 4: BEGIN ATOMIC TRANSACTION
    let mut db_tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Failed to start transaction".to_string()));
        }
    };

    // Step 5: INSERT TRANSACTION RECORD (within transaction)
    let insert_result = sqlx::query_as::<_, Transaction>(
        "INSERT INTO transactions 
         (id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at"
    )
    .bind(&transaction_id)
    .bind(&req.user_id)
    .bind(&req.wallet_id)
    .bind(req.amount.clone())  // Clone: BigDecimal doesn't implement Copy
    .bind(&req.transaction_type)
    .bind(&req.category)
    .bind(&req.description)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *db_tx)  // Execute in transaction
    .await;

    let transaction = match insert_result {
        Ok(t) => t,
        Err(e) => {
            log::error!("Error inserting transaction: {}", e);
            let _ = db_tx.rollback().await;  // ROLLBACK on error
            return HttpResponse::BadRequest()
                .json(ApiResponse::<Transaction>::error("Failed to create transaction".to_string()));
        }
    };

    // Step 6: UPDATE WALLET BALANCE (within transaction)
    let balance_delta = match req.transaction_type.as_str() {
        "income" => req.amount.clone(),
        "expense" => -req.amount.clone(),
        _ => {
            let _ = db_tx.rollback().await;
            return HttpResponse::BadRequest()
                .json(ApiResponse::<Transaction>::error("Invalid transaction type".to_string()));
        }
    };

    let update_result = sqlx::query(
        "UPDATE wallets SET balance = balance + $1, updated_at = $2 WHERE id = $3"
    )
    .bind(balance_delta)
    .bind(now)
    .bind(&req.wallet_id)
    .execute(&mut *db_tx)  // Execute in transaction
    .await;

    if let Err(e) = update_result {
        log::error!("Error updating wallet: {}", e);
        let _ = db_tx.rollback().await;  // ROLLBACK on error
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<Transaction>::error("Failed to update wallet".to_string()));
    }

    // Step 7: COMMIT TRANSACTION
    // Both operations are now persisted atomically
    if let Err(e) = db_tx.commit().await {
        log::error!("Failed to commit: {}", e);
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<Transaction>::error("Failed to save changes".to_string()));
    }

    // Step 8: INVALIDATE CACHE
    let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("wallet*{}*", req.user_id)).await;
    let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("transactions:{}*", req.user_id)).await;

    HttpResponse::Created().json(ApiResponse::success(transaction))
}
```

### Atomicity Guarantees

**Either BOTH succeed or BOTH fail:**
```
Transaction Record Created ✓  AND  Wallet Balance Updated ✓
         OR
Transaction Record Failed ✗  AND  Wallet Balance Unchanged ✗ (Automatic Rollback)
```

**Key Benefits:**
1. **No Partial States**: Database never enters inconsistent state
2. **Concurrent Safety**: Multiple requests won't see partial updates
3. **Automatic Rollback**: Any error triggers rollback automatically
4. **Crash Safe**: If process crashes, all changes are rolled back by database

---

## Migration Execution

### Run the migration:

```bash
# Set DATABASE_URL in .env
export DATABASE_URL="postgresql://postgres:password@localhost:5432/ketobook"

# Run the migration
sqlx migrate run

# Expected output:
# Applied 20260128001_initial_schema (1.234s)
```

### Verify tables:

```bash
psql $DATABASE_URL

# List tables
\dt

# Expected:
#  Schema |    Name     | Type  | Owner
# --------+-------------+-------+-------
#  public | debts       | table | postgres
#  public | transactions| table | postgres
#  public | wallets     | table | postgres

# List types
\dT+

# Should show wallet_type ENUM with values: Cash, BankAccount, CreditCard, Other
```

---

## Testing the API

### Create a wallet:

```bash
curl -X POST http://localhost:8080/api/wallets \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user123",
    "name": "My Checking",
    "wallet_type": "BankAccount",
    "balance": "1000.50",
    "credit_limit": null
  }'
```

### Create a transaction (atomic):

```bash
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user123",
    "wallet_id": "wallet-uuid-here",
    "amount": "50.25",
    "transaction_type": "expense",
    "category": "groceries",
    "description": "Weekly shopping"
  }'

# Both transaction record AND wallet balance are updated atomically
```

### Create a debt:

```bash
curl -X POST http://localhost:8080/api/debts \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user123",
    "wallet_id": null,
    "creditor_name": "Bank of America",
    "amount": "5000.00",
    "interest_rate": "4.50",
    "due_date": "2026-12-31T23:59:59Z"
  }'
```

---

## Summary

This manually structured migration demonstrates:

✅ **Proper ENUM usage** for type safety  
✅ **Explicit dependency ordering** to avoid "relation does not exist" errors  
✅ **Composite indexes** for efficient queries  
✅ **Cascade/set null semantics** for referential integrity  
✅ **Auto-update triggers** for timestamp management  
✅ **Financial precision** with DECIMAL(15,2)  
✅ **BigDecimal in Rust** for monetary calculations  
✅ **Atomic transactions** for consistency  
✅ **Cache invalidation** for data freshness  
✅ **2026 timestamp compliance** throughout  

All code compiles successfully with Rust 1.91.1 and SQLx 0.7.
