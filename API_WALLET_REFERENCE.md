# KetoBook Multi-Wallet API - Quick Reference

## Financial Precision & Credit Card Support

**All monetary values use BigDecimal for precision:**
- Accurate to 2 decimal places (cents)
- No floating-point rounding errors
- Safe for financial calculations

**Credit Card Wallet Type:**
- `balance` field represents current debt (0 = no debt, limit = fully used)
- `credit_limit` field specifies maximum spending
- Available credit = credit_limit - balance
- Transactions validated against available credit

## Wallet Management Endpoints

### Create Bank Account Wallet
```bash
POST /api/wallets
Content-Type: application/json

{
  "user_id": "user123",
  "name": "My Checking Account",
  "wallet_type": "BankAccount",
  "balance": "5000.00"
}

# Response: 201 Created
{
  "success": true,
  "data": {
    "id": "wallet-uuid-1",
    "user_id": "user123",
    "name": "My Checking Account",
    "balance": "5000.00",
    "credit_limit": null,
    "wallet_type": "BankAccount",
    "created_at": "2026-01-28T12:00:00Z",
    "updated_at": "2026-01-28T12:00:00Z"
  }
}
```

### Create Credit Card Wallet (NEW)
```bash
POST /api/wallets
Content-Type: application/json

{
  "user_id": "user123",
  "name": "Visa Premium",
  "wallet_type": "CreditCard",
  "balance": "2500.00",
  "credit_limit": "10000.00"
}

# Response: 201 Created
{
  "success": true,
  "data": {
    "id": "wallet-uuid-cc",
    "user_id": "user123",
    "name": "Visa Premium",
    "balance": "2500.00",           # Current debt
    "credit_limit": "10000.00",     # Credit limit
    "wallet_type": "CreditCard",
    "created_at": "2026-01-28T12:00:00Z",
    "updated_at": "2026-01-28T12:00:00Z"
  }
}
```

**Note:** Available credit = 10000.00 - 2500.00 = 7500.00

### Get All Wallets
```bash
GET /api/wallets/user/user123

# Response: 200 OK
{
  "success": true,
  "data": [
    {
      "id": "wallet-uuid-1",
      "user_id": "user123",
      "name": "Checking",
      "balance": "5000.00",
      "credit_limit": null,
      "wallet_type": "BankAccount",
      "created_at": "2026-01-28T10:00:00Z",
      "updated_at": "2026-01-28T12:00:00Z"
    },
    {
      "id": "wallet-uuid-2",
      "user_id": "user123",
      "name": "Visa Premium",
      "balance": "2500.00",
      "credit_limit": "10000.00",
      "wallet_type": "CreditCard",
      "created_at": "2026-01-28T11:00:00Z",
      "updated_at": "2026-01-28T11:30:00Z"
    },
    {
      "id": "wallet-uuid-3",
      "user_id": "user123",
      "name": "Cash Wallet",
      "balance": "200.00",
      "credit_limit": null,
      "wallet_type": "Cash",
      "created_at": "2026-01-28T11:00:00Z",
      "updated_at": "2026-01-28T11:30:00Z"
    }
  ]
}
```

### Get Specific Wallet
```bash
GET /api/wallets/user123/wallet-uuid-cc

# Response: 200 OK
{
  "success": true,
  "data": {
    "id": "wallet-uuid-cc",
    "user_id": "user123",
    "name": "Visa Premium",
    "balance": "2500.00",
    "credit_limit": "10000.00",
    "wallet_type": "CreditCard",
    "created_at": "2026-01-28T10:00:00Z",
    "updated_at": "2026-01-28T12:00:00Z"
  }
}
```

### Update Wallet
```bash
PUT /api/wallets/user123/wallet-uuid-1
Content-Type: application/json

{
  "name": "Primary Checking",
  "balance": "5250.00"
}

# Response: 200 OK
{
  "success": true,
  "data": {
    "id": "wallet-uuid-1",
    "user_id": "user123",
    "name": "Primary Checking",
    "balance": "5250.00",
    "credit_limit": null,
    "wallet_type": "BankAccount",
    "created_at": "2026-01-28T10:00:00Z",
```
    "updated_at": "2026-01-28T12:30:00Z"
  }
}
```

### Delete Wallet
```bash
DELETE /api/wallets/user123/wallet-uuid-1

# Response: 204 No Content
# (Also cascades: deletes all transactions associated with this wallet)
```

## Transaction Endpoints (Enhanced with Atomic Operations)

### Create Transaction with Balance Validation

**Standard Wallet (BankAccount/Cash):**
```bash
POST /api/transactions
Content-Type: application/json

{
  "user_id": "user123",
  "wallet_id": "wallet-uuid-1",
  "amount": "50.00",
  "transaction_type": "expense",
  "category": "groceries",
  "description": "Weekly shopping"
}

# Response: 201 Created
{
  "success": true,
  "data": {
    "id": "txn-uuid-1",
    "user_id": "user123",
    "wallet_id": "wallet-uuid-1",
    "amount": "50.00",
    "transaction_type": "expense",
    "category": "groceries",
    "description": "Weekly shopping",
    "created_at": "2026-01-28T12:30:00Z",
    "updated_at": "2026-01-28T12:30:00Z"
  }
}

# Wallet balance automatically updated (atomic transaction):
# wallet-uuid-1 balance: 5000.00 - 50.00 = 4950.00
```

**Credit Card Wallet (Available Credit Validation):**
```bash
POST /api/transactions
Content-Type: application/json

{
  "user_id": "user123",
  "wallet_id": "wallet-uuid-cc",
  "amount": "500.00",
  "transaction_type": "expense",
  "category": "dining",
  "description": "Dinner"
}

# Response: 201 Created
{
  "success": true,
  "data": {
    "id": "txn-uuid-cc-1",
    "user_id": "user123",
    "wallet_id": "wallet-uuid-cc",
    "amount": "500.00",
    "transaction_type": "expense",
    "category": "dining",
    "description": "Dinner",
    "created_at": "2026-01-28T13:00:00Z",
    "updated_at": "2026-01-28T13:00:00Z"
  }
}

# Credit card balance automatically updated (atomic transaction):
# wallet-uuid-cc balance: 2500.00 + 500.00 = 3000.00 (more debt)
# Available credit: 10000.00 - 3000.00 = 7000.00
```

**Insufficient Funds/Credit Error:**
```bash
POST /api/transactions
Content-Type: application/json

{
  "user_id": "user123",
  "wallet_id": "wallet-uuid-cc",
  "amount": "8000.00",          # More than available credit (7000.00)
  "transaction_type": "expense",
  "category": "shopping",
  "description": "Large purchase"
}

# Response: 400 Bad Request
{
  "success": false,
  "error": "Insufficient available credit. Required: 8000.00, Available: 7000.00"
}
# Transaction REJECTED - wallet balance unchanged (atomic rollback)
```

### Update Transaction (Can Change Wallet & Amount)
```bash
PUT /api/transactions/user123/txn-uuid-1
Content-Type: application/json

{
  "wallet_id": "wallet-uuid-cc",    # Move to credit card
  "amount": "75.00",                 # Update amount
  "category": "dining",
  "description": "Lunch at restaurant"
}

# Response: 200 OK
{
  "success": true,
  "data": {
    "id": "txn-uuid-1",
    "user_id": "user123",
    "wallet_id": "wallet-uuid-cc",
    "amount": "75.00",
    "transaction_type": "expense",
    "category": "dining",
    "description": "Lunch at restaurant",
    "created_at": "2026-01-28T12:30:00Z",
    "updated_at": "2026-01-28T12:45:00Z"
  }
}

# Wallet balances updated atomically:
# wallet-uuid-1: 4950.00 + 50.00 = 5000.00 (old amount reversed)
# wallet-uuid-cc: 3000.00 + 75.00 = 3075.00 (new amount applied)
```

### Delete Transaction (Reverses Impact)
```bash
DELETE /api/transactions/user123/txn-uuid-1

# Response: 204 No Content

# Wallet balance atomically reversed:
# wallet-uuid-cc: 3075.00 - 75.00 = 3000.00 (expense reversed)
```

## Wallet Types

Available wallet types:
- **Cash** - Physical cash wallet
- **BankAccount** - Bank savings/checking account
- **CreditCard** - Credit card with credit_limit field
- **Other** - Any other type of account

## Atomic Transaction Guarantees

### What are Atomic Transactions?

All transaction operations (create, update, delete) use PostgreSQL atomic transactions (BEGIN/COMMIT) to ensure data consistency:

- **All-or-Nothing:** Either the entire operation succeeds or it completely rolls back
- **No Partial Updates:** Never a situation where transaction is created but balance isn't updated
- **Consistency:** Wallet balance always reflects transaction history
- **Safety:** Automatic rollback on any error

### Create Transaction Flow
```
1. BEGIN database transaction
2. Fetch wallet (lock for consistency)
3. Validate:
   - Wallet exists and belongs to user
   - Amount > 0
   - Transaction type is "income" or "expense"
   - For CreditCard: available_credit >= amount
   - For other: balance >= amount (expenses only)
4. IF validation fails:
   - ROLLBACK automatically
   - Return error to client
   - Wallet unchanged ✓
5. IF validation passes:
   - INSERT transaction record
   - UPDATE wallet balance (atomic with transaction insert)
   - COMMIT (all changes permanent)
   - Return success to client
```

### Wallet Balance Validation

**Regular Wallets (Cash, BankAccount, Other):**
- Income transactions: Always allowed (balance increases)
- Expense transactions: Only allowed if `balance >= amount`
- Prevents negative balance

**Credit Card Wallets:**
- Income transactions: Always allowed (reduces debt)
- Expense transactions: Only allowed if `available_credit >= amount`
  - Available credit = `credit_limit - balance`
  - Prevents exceeding credit limit

### Update Transaction Flow
```
1. BEGIN database transaction
2. Fetch current transaction
3. IF wallet changed OR amount changed:
   - Reverse old wallet balance (undo original transaction)
   - Validate new wallet/amount combination
   - IF validation fails: ROLLBACK, return error
   - Apply new balance to new wallet
4. Commit all balance changes atomically
5. Invalidate relevant caches
```

### Delete Transaction Flow
```
1. BEGIN database transaction
2. Fetch transaction details
3. Reverse balance on wallet:
   - If income: subtract amount
   - If expense: add amount
4. Delete transaction record
5. COMMIT (balance reversal + deletion atomic)
6. Invalidate caches
```

### Cache Invalidation

After any transaction operation:
- Invalidate wallet-specific cache: `wallet:{user_id}:{wallet_id}`
- Invalidate user wallet list: `wallets:{user_id}`
- Invalidate user transactions: `transactions:{user_id}`
- Next API call will fetch fresh data from database

## Example Workflow

### Step 1: Create Wallets
```bash
# Create checking account
curl -X POST http://localhost:8080/api/wallets \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user123",
    "name": "Checking",
    "wallet_type": "BankAccount",
    "balance": 5000
  }'
# Response: wallet-id-1 with balance 5000

# Create cash wallet
curl -X POST http://localhost:8080/api/wallets \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user123",
    "name": "Cash",
    "wallet_type": "Cash",
    "balance": 200
  }'
# Response: wallet-id-2 with balance 200
```

### Step 2: Create Income Transaction
```bash
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user123",
    "wallet_id": "wallet-id-1",
    "amount": 1000,
    "transaction_type": "income",
    "category": "salary",
    "description": "Monthly salary"
  }'
# Checking balance: 5000 + 1000 = 6000
```

### Step 3: Create Expense Transaction
```bash
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user123",
    "wallet_id": "wallet-id-1",
    "amount": 75.50,
    "transaction_type": "expense",
    "category": "groceries",
    "description": "Supermarket"
  }'
# Checking balance: 6000 - 75.50 = 5924.50
```

### Step 4: Move Transaction Between Wallets
```bash
curl -X PUT http://localhost:8080/api/transactions/user123/txn-id \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_id": "wallet-id-2"
  }'
# Checking balance: 5924.50 + 75.50 = 6000
# Cash balance: 200 - 75.50 = 124.50
```

### Step 5: View All Wallets and Balances
```bash
curl http://localhost:8080/api/wallets/user/user123

# Shows:
# - Checking: 6000.00
# - Cash: 124.50
```

## Data Flow on Transaction Create

1. Validate wallet exists and belongs to user
2. Begin database transaction
3. Insert transaction record
4. Update wallet balance:
   - If income: balance += amount
   - If expense: balance -= amount
5. Commit all changes atomically
6. Invalidate Redis cache for wallets and transactions
7. Return created transaction

## Data Flow on Wallet Delete

1. Delete wallet record
2. Cascade: All associated transactions deleted
3. Wallet balance no longer tracked
4. Invalidate Redis cache
5. Return 204 No Content

## Error Responses

### Wallet Not Found
```bash
{
  "success": false,
  "data": null,
  "error": "Wallet not found or doesn't belong to user"
}
```

### Invalid Transaction Type
```bash
{
  "success": false,
  "data": null,
  "error": "Invalid transaction type"
}
```

### Database Error
```bash
{
  "success": false,
  "data": null,
  "error": "Failed to create wallet"
}
```

## Caching Behavior

- Wallet lists cached for 1 hour
- Individual wallet details cached for 1 hour
- Cache automatically invalidated on:
  - Create transaction
  - Update transaction
  - Delete transaction
  - Create wallet
  - Update wallet
  - Delete wallet

## Performance Tips

1. **Batch Operations**: Create multiple transactions in one session to reduce API calls
2. **Cache Warming**: List wallets once at app start to cache them
3. **Wallet Organization**: Use meaningful wallet names for easy identification
4. **Transaction Categories**: Use consistent categories for better analytics
5. **Balance Reconciliation**: Periodically verify wallet balances match your records

## Migration to Multi-Wallet

If migrating from single-wallet system:

1. Create default wallet for each user
2. Move existing transactions to default wallet
3. Update application to use wallet_id
4. Test transaction creation/updates
5. Verify wallet balances

Example migration script:
```sql
-- Create default wallet for each user
INSERT INTO wallets (id, user_id, name, wallet_type, balance, created_at, updated_at)
SELECT 
  gen_random_uuid()::text,
  user_id,
  'Default Wallet',
  'Other'::wallet_type,
  0,
  CURRENT_TIMESTAMP,
  CURRENT_TIMESTAMP
FROM (SELECT DISTINCT user_id FROM transactions) users;

-- Update transactions to reference wallet
UPDATE transactions t
SET wallet_id = w.id
FROM wallets w
WHERE t.user_id = w.user_id
AND t.wallet_id IS NULL;
```
