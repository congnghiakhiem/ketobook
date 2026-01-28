# Models Refactoring - Modular Structure (2026-01-28)

## Overview

The `src/models.rs` file has been refactored into a modular directory structure for better organization and maintainability.

## Directory Structure

```
src/
├── models/
│   ├── mod.rs           (Module exports and common types)
│   ├── wallet.rs        (Wallet and WalletType definitions)
│   ├── transaction.rs   (Transaction definitions)
│   └── debt.rs          (Debt definitions)
├── main.rs              (Entry point - no changes to imports)
├── wallets.rs           (Wallet handlers - imports from models)
├── transactions.rs      (Transaction handlers - imports from models)
├── debts.rs             (Debt handlers - imports from models)
├── cache.rs
├── config.rs
└── db.rs
```

## File Organization

### src/models/mod.rs

**Purpose:** Central module export point for all model types.

```rust
pub mod wallet;
pub use wallet::{Wallet, WalletType, CreateWalletRequest, UpdateWalletRequest};

pub mod transaction;
pub use transaction::{Transaction, CreateTransactionRequest, UpdateTransactionRequest};

pub mod debt;
pub use debt::{Debt, CreateDebtRequest, UpdateDebtRequest};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> { ... }
```

**Exports:**
- All types from wallet, transaction, and debt modules
- Shared `ApiResponse<T>` generic response wrapper

**Benefits:**
- Single import point: `use crate::models::{Wallet, Transaction, Debt};`
- Alternative imports: `use crate::models::wallet::Wallet;`
- Clear re-exports avoid `pub use` pollution

---

### src/models/wallet.rs

**Size:** ~105 lines  
**Purpose:** Wallet model and wallet type definitions

**Contains:**

```rust
pub enum WalletType {
    Cash,
    BankAccount,
    CreditCard,
    Other,
}

impl WalletType {
    pub fn as_str(&self) -> &'static str { ... }
    pub fn from_str(s: &str) -> Option<Self> { ... }
    pub fn is_credit_card(&self) -> bool { ... }
}

pub struct Wallet {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub balance: BigDecimal,
    pub credit_limit: Option<BigDecimal>,
    pub wallet_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Wallet {
    pub fn wallet_type_enum(&self) -> Option<WalletType> { ... }
    pub fn available_balance(&self) -> BigDecimal { ... }
}

pub struct CreateWalletRequest { ... }
pub struct UpdateWalletRequest { ... }
```

**Key Features:**
- `sqlx::FromRow` for direct database deserialization
- `BigDecimal` for balance and credit_limit (financial precision)
- Helper methods for wallet type conversion
- Available balance calculation (handles credit cards specially)

**Credit Card Semantics:**
```
balance = current debt
available_balance = credit_limit - balance
```

---

### src/models/transaction.rs

**Size:** ~43 lines  
**Purpose:** Transaction model definitions

**Contains:**

```rust
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub wallet_id: String,              // Required (not Option)
    pub amount: BigDecimal,             // Always positive
    pub transaction_type: String,       // "income" or "expense"
    pub category: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CreateTransactionRequest { ... }
pub struct UpdateTransactionRequest { ... }
```

**Key Features:**
- `wallet_id` is required (String, not Option)
- `amount` always positive; type determines operation
- `sqlx::FromRow` for database deserialization
- `BigDecimal` for amount field

---

### src/models/debt.rs

**Size:** ~45 lines  
**Purpose:** Debt model definitions

**Contains:**

```rust
pub struct Debt {
    pub id: String,
    pub user_id: String,
    pub wallet_id: Option<String>,      // Optional FK (SET NULL on delete)
    pub creditor_name: String,
    pub amount: BigDecimal,             // Financial precision
    pub interest_rate: BigDecimal,      // Percentage, financial precision
    pub due_date: Option<DateTime<Utc>>,
    pub status: String,                 // "active", "paid", "cancelled"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CreateDebtRequest { ... }
pub struct UpdateDebtRequest { ... }
```

**Key Features:**
- `wallet_id` is optional (user-level debt tracking)
- `amount` and `interest_rate` use `BigDecimal`
- `status` supports three states: active, paid, cancelled
- `sqlx::FromRow` for database deserialization

---

## Import Examples

### Before (Single File)
```rust
use crate::models::{Wallet, Transaction, Debt, ApiResponse};
```

### After (Modular)

**Option 1: Import from mod.rs (recommended for handlers)**
```rust
use crate::models::{Wallet, Transaction, Debt, ApiResponse};
```

**Option 2: Import from specific modules**
```rust
use crate::models::wallet::{Wallet, WalletType};
use crate::models::transaction::Transaction;
use crate::models::debt::Debt;
use crate::models::ApiResponse;
```

**Option 3: Module path (more explicit)**
```rust
use crate::models;

let wallet: models::Wallet = ...;
let transaction: models::transaction::Transaction = ...;
```

---

## Compilation Status

✅ **Code compiles successfully** with new modular structure

Warnings (unchanged from before):
- Unused functions in cache.rs, db.rs, wallets.rs
- Unused methods in wallet.rs
- All warnings are pre-existing and unrelated to refactoring

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.22s
```

---

## Migration Path

### For Existing Code:

**No changes required!** The module exports in `mod.rs` re-export all public types, so existing imports continue to work:

```rust
// This still works exactly the same:
use crate::models::{Wallet, Transaction, ApiResponse};
```

### For New Code:

You can now use more specific imports if desired:

```rust
// More explicit, shows where types come from:
use crate::models::wallet::Wallet;
use crate::models::transaction::Transaction;
use crate::models::debt::Debt;
```

---

## Benefits of This Structure

### Organization
- **Clear separation of concerns**: Each model type in its own file
- **Easier navigation**: Less scrolling, clearer file purposes
- **Logical grouping**: Related types and implementations together

### Maintainability
- **Smaller files**: Easier to read and understand
- **Isolated changes**: Modifications to Wallet don't affect Transaction code
- **Testing**: Can test models in isolation if needed

### Scalability
- **Future-proof**: Easy to add new model types (e.g., `category.rs`, `tag.rs`)
- **Growth friendly**: Adding fields or methods stays contained
- **Plugin-ready**: Can easily add optional modules later

### Documentation
- **Self-documenting**: File names describe contents
- **Focused docs**: Module-level documentation for each type
- **Clear relationships**: Dependencies between types are explicit

---

## Database Schema Mapping

Each model maps to a database table:

### wallet.rs ↔ wallets table
```
Wallet struct fields          →  wallets table columns
id                            →  id (UUID)
user_id                       →  user_id (VARCHAR)
name                          →  name (VARCHAR)
balance                       →  balance (DECIMAL 15,2)
credit_limit                  →  credit_limit (DECIMAL 15,2)
wallet_type                   →  wallet_type (wallet_type ENUM)
created_at                    →  created_at (TIMESTAMP WITH TIME ZONE)
updated_at                    →  updated_at (TIMESTAMP WITH TIME ZONE)
```

### transaction.rs ↔ transactions table
```
Transaction struct fields     →  transactions table columns
id                            →  id (UUID)
user_id                       →  user_id (VARCHAR)
wallet_id                     →  wallet_id (VARCHAR, FK to wallets)
amount                        →  amount (DECIMAL 15,2)
transaction_type              →  transaction_type (VARCHAR)
category                      →  category (VARCHAR)
description                   →  description (TEXT)
created_at                    →  created_at (TIMESTAMP WITH TIME ZONE)
updated_at                    →  updated_at (TIMESTAMP WITH TIME ZONE)
```

### debt.rs ↔ debts table
```
Debt struct fields            →  debts table columns
id                            →  id (UUID)
user_id                       →  user_id (VARCHAR)
wallet_id                     →  wallet_id (VARCHAR, FK to wallets, nullable)
creditor_name                 →  creditor_name (VARCHAR)
amount                        →  amount (DECIMAL 15,2)
interest_rate                 →  interest_rate (DECIMAL 5,2)
due_date                      →  due_date (TIMESTAMP WITH TIME ZONE)
status                        →  status (VARCHAR)
created_at                    →  created_at (TIMESTAMP WITH TIME ZONE)
updated_at                    →  updated_at (TIMESTAMP WITH TIME ZONE)
```

---

## Type Safety

All models use:
- ✅ `sqlx::FromRow` - Type-safe database deserialization
- ✅ `BigDecimal` - Exact financial calculations (no floating-point errors)
- ✅ `DateTime<Utc>` - Timezone-aware timestamps
- ✅ `Option<T>` - Proper handling of nullable database columns
- ✅ Serde derive - JSON serialization/deserialization

---

## File Changes Summary

| File | Action | Details |
|------|--------|---------|
| `src/models.rs` | Deleted | Replaced by `src/models/` directory |
| `src/models/mod.rs` | Created | Module exports and ApiResponse |
| `src/models/wallet.rs` | Created | Wallet and WalletType types |
| `src/models/transaction.rs` | Created | Transaction types |
| `src/models/debt.rs` | Created | Debt types |
| `src/main.rs` | No change | `mod models;` now points to directory |
| `src/wallets.rs` | No change | Imports work via mod.rs re-exports |
| `src/transactions.rs` | No change | Imports work via mod.rs re-exports |
| `src/debts.rs` | No change | Imports work via mod.rs re-exports |

---

## Example: Using the Refactored Models

### In a Handler (src/wallets.rs)

```rust
use crate::models::{Wallet, CreateWalletRequest, UpdateWalletRequest, ApiResponse};

pub async fn create_wallet(
    req: web::Json<CreateWalletRequest>,
    db: web::Data<PgPool>,
) -> HttpResponse {
    // wallet_type field is an enum
    let wallet_type = req.wallet_type.as_str();
    
    // Create wallet with BigDecimal precision
    let wallet: Wallet = sqlx::query_as(
        "INSERT INTO wallets (...) VALUES (...) RETURNING ..."
    )
    .bind(req.balance)  // BigDecimal automatically handled by SQLx
    .fetch_one(db.get_ref())
    .await
    .unwrap();
    
    // Use available_balance helper method
    let available = wallet.available_balance();
    
    HttpResponse::Created().json(ApiResponse::success(wallet))
}
```

### In a Handler (src/transactions.rs)

```rust
use crate::models::{Transaction, CreateTransactionRequest, ApiResponse};

pub async fn create_transaction(
    req: web::Json<CreateTransactionRequest>,
    db: web::Data<PgPool>,
) -> HttpResponse {
    // wallet_id is required
    let wallet_id = &req.wallet_id;
    
    // amount uses BigDecimal
    let amount = req.amount.clone();
    
    let transaction: Transaction = sqlx::query_as(
        "INSERT INTO transactions (...) VALUES (...) RETURNING ..."
    )
    .bind(amount)  // BigDecimal automatically handled
    .fetch_one(db.get_ref())
    .await
    .unwrap();
    
    HttpResponse::Created().json(ApiResponse::success(transaction))
}
```

---

## Verification

Run compilation check:
```bash
cargo check
```

Expected output:
```
Checking ketobook v0.1.0 (C:\Projects\ketobook)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.22s
```

Run with warnings:
```bash
cargo build
```

All existing warnings remain (pre-existing unused code), no new warnings introduced.

---

## Summary

✅ **Refactoring complete:**
- Modular directory structure for better organization
- Clear separation: wallet.rs, transaction.rs, debt.rs
- Central exports via mod.rs
- All types maintain backward compatibility
- Code compiles successfully
- No breaking changes to existing imports

✅ **All 2026 features preserved:**
- BigDecimal for financial precision
- DateTime<Utc> for timezone-aware timestamps
- Atomic transaction support
- Credit card balance semantics
- Comprehensive model documentation

**Status:** Ready for production use.
