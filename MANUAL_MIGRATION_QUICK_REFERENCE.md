# KetoBook Manual Migration - Quick Reference (2026)

## Files Provided

### 1. Migration File
**Location:** `migrations/20260128001_initial_schema.sql` (252 lines)

**Contents:**
- ✅ wallet_type ENUM (Cash, BankAccount, CreditCard, Other)
- ✅ wallets table (id, user_id, name, balance, credit_limit, wallet_type, timestamps)
- ✅ transactions table (id, user_id, wallet_id FK, amount, transaction_type, category, description, timestamps)
- ✅ debts table (id, user_id, wallet_id FK, creditor_name, amount, interest_rate, due_date, status, timestamps)
- ✅ v_transaction_summary view (aggregations by user/wallet/type)
- ✅ v_debt_summary view (aggregations by user/status)
- ✅ v_wallet_summary view (with available_balance calculation)
- ✅ All indexes for efficient queries
- ✅ Auto-update triggers for timestamps
- ✅ Comprehensive comments on all tables/columns

**Execution Order:**
```
1. wallet_type ENUM ← No dependencies
2. wallets table ← Depends on wallet_type ENUM
3. transactions table ← Depends on wallets (FK)
4. debts table ← Depends on wallets (FK)
5. Views & Triggers ← Depend on base tables
```

---

### 2. Rust Models (src/models.rs)
All models use:
- ✅ `sqlx::FromRow` derive for automatic deserialization
- ✅ `BigDecimal` for all monetary fields (DECIMAL 15,2 in DB)
- ✅ `DateTime<Utc>` for timestamps (TIMESTAMP WITH TIME ZONE in DB)
- ✅ `Option<T>` for nullable columns

**Key Models:**
```rust
#[derive(sqlx::FromRow)]
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

#[derive(sqlx::FromRow)]
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub wallet_id: String,              // Required (not Option)
    pub amount: BigDecimal,             // Use BigDecimal, not f64!
    pub transaction_type: String,
    pub category: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct Debt {
    pub id: String,
    pub user_id: String,
    pub wallet_id: Option<String>,      // Optional FK to wallets
    pub creditor_name: String,
    pub amount: BigDecimal,             // Use BigDecimal, not f64!
    pub interest_rate: BigDecimal,      // Use BigDecimal, not f64!
    pub due_date: Option<DateTime<Utc>>,
    pub status: String,                 // "active", "paid", "cancelled"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

### 3. Atomic Transaction Handler (src/transactions.rs)
Complete pattern showing:
- ✅ `db.begin().await` - Start transaction
- ✅ All operations within `&mut *db_tx` - Execute in transaction
- ✅ `db_tx.rollback().await` - Automatic rollback on error
- ✅ `db_tx.commit().await` - Persist both changes atomically
- ✅ `BigDecimal.clone()` - Required for binding (no Copy impl)

**Pattern:**
```rust
// Step 1: Validate input
// Step 2: Fetch wallet
// Step 3: BEGIN TRANSACTION
let mut db_tx = db.begin().await?;

// Step 4: INSERT transaction record
sqlx::query(...).execute(&mut *db_tx).await?;

// Step 5: UPDATE wallet balance
sqlx::query(...).execute(&mut *db_tx).await?;

// Step 6: COMMIT
db_tx.commit().await?;

// Step 7: Invalidate cache
```

**Atomicity Guarantee:**
- Transaction created ✓ AND Wallet updated ✓
- OR Both rollback on error ✗

---

### 4. Documentation (MANUAL_MIGRATION_STRUCTURE.md)
Comprehensive guide covering:
- ✅ Migration overview & execution order
- ✅ Complete schema for each table
- ✅ ENUM types and constraints
- ✅ Foreign key semantics (CASCADE DELETE vs SET NULL)
- ✅ Index strategies for performance
- ✅ Trigger implementations
- ✅ Rust model integration
- ✅ Step-by-step atomic transaction example
- ✅ API testing examples
- ✅ Credit card balance semantics

---

## Key Design Decisions

### 1. Monetary Fields: DECIMAL(15, 2)
```sql
-- Precision: 15 total digits, 2 decimal places
-- Handles: $0.01 to $999,999,999,999.99
-- Why: Exact calculations (no floating-point errors)
```

### 2. Timestamps: TIMESTAMP WITH TIME ZONE
```sql
-- All timestamps in UTC (2026)
-- Handles: Timezone-aware calculations
-- Why: Consistent across time zones
```

### 3. Foreign Key Semantics
```sql
-- Transactions -> Wallets: CASCADE DELETE
-- (Delete wallet → Delete all associated transactions)

-- Debts -> Wallets: SET NULL
-- (Delete wallet → Keep debt but clear wallet_id)
```

### 4. Credit Card Balance Semantics
```rust
// For CreditCard wallet_type:
balance = current debt (0 = no debt, limit = fully used)
available_credit = credit_limit - balance

// Example:
credit_limit = 5000.00
balance = 1500.00  (current debt/usage)
available_credit = 3500.00 (can spend)
```

### 5. Atomic Transactions
```rust
// Both succeed or both fail:
INSERT transaction record ✓
UPDATE wallet balance ✓
// OR (on any error):
ROLLBACK all changes ✗
```

---

## Migration Execution

### Option 1: Use New Manual Migration (Recommended)
```bash
# Remove old migration (optional)
rm migrations/20260101_create_all_tables.sql

# Run new migration
sqlx migrate run

# Expected: Applied 20260128001_initial_schema (1.234s)
```

### Option 2: Keep Both Migrations
```bash
# SQLx will run both:
sqlx migrate run

# This may cause errors if schemas conflict
# Solution: Revert old one first:
sqlx migrate revert --all
# Then remove: rm migrations/20260101_create_all_tables.sql
# Then run: sqlx migrate run
```

---

## Testing

### 1. Create Wallet
```bash
curl -X POST http://localhost:8080/api/wallets \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user123",
    "name": "My Checking",
    "wallet_type": "BankAccount",
    "balance": "1000.00",
    "credit_limit": null
  }'
```

### 2. Create Transaction (Atomic)
```bash
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user123",
    "wallet_id": "wallet-uuid",
    "amount": "50.00",
    "transaction_type": "expense",
    "category": "groceries",
    "description": "Weekly shopping"
  }'
```
✅ Both transaction record created AND wallet balance updated atomically

### 3. Create Debt
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

## Important Notes

⚠️ **Two Migration Files Exist:**
- `migrations/20260101_create_all_tables.sql` (old consolidated)
- `migrations/20260128001_initial_schema.sql` (new manual)

**Recommendation:** Remove old migration before running to avoid conflicts.

✅ **Code Compiles:** Rust 1.91.1 with all warnings clean  
✅ **Financial Precision:** BigDecimal throughout (no f64 for money)  
✅ **Atomicity:** All transactions use BEGIN/COMMIT/ROLLBACK  
✅ **2026 Compliant:** All timestamps UTC, schema designed for 2026  

---

## File Locations

```
c:\Projects\ketobook\
├── migrations/
│   ├── 20260101_create_all_tables.sql (old - consider removing)
│   └── 20260128001_initial_schema.sql (new - use this)
├── src/
│   ├── models.rs (updated with correct types)
│   └── transactions.rs (updated with atomic pattern docs)
├── MIGRATIONS.md (old, may need update)
└── MANUAL_MIGRATION_STRUCTURE.md (new - comprehensive guide)
```

---

**Summary:** Complete manual migration structure with explicit dependency ordering, comprehensive documentation, and Rust integration for atomic transactions. All 2026 compliant with financial precision using BigDecimal and proper timezone handling.
