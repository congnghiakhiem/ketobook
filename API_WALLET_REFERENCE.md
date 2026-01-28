# KetoBook Multi-Wallet API - Quick Reference

## Wallet Management Endpoints

### Create Wallet
```bash
POST /api/wallets
Content-Type: application/json

{
  "user_id": "user123",
  "name": "My Checking Account",
  "wallet_type": "BankAccount",
  "balance": 5000.00
}

# Response: 201 Created
{
  "success": true,
  "data": {
    "id": "wallet-uuid-1",
    "user_id": "user123",
    "name": "My Checking Account",
    "balance": 5000.00,
    "wallet_type": "BankAccount",
    "created_at": "2026-01-28T12:00:00Z",
    "updated_at": "2026-01-28T12:00:00Z"
  }
}
```

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
      "balance": 5000.00,
      "wallet_type": "BankAccount",
      "created_at": "2026-01-28T10:00:00Z",
      "updated_at": "2026-01-28T12:00:00Z"
    },
    {
      "id": "wallet-uuid-2",
      "user_id": "user123",
      "name": "Cash Wallet",
      "balance": 200.00,
      "wallet_type": "Cash",
      "created_at": "2026-01-28T11:00:00Z",
      "updated_at": "2026-01-28T11:30:00Z"
    }
  ]
}
```

### Get Specific Wallet
```bash
GET /api/wallets/user123/wallet-uuid-1

# Response: 200 OK
{
  "success": true,
  "data": {
    "id": "wallet-uuid-1",
    "user_id": "user123",
    "name": "Checking",
    "balance": 5000.00,
    "wallet_type": "BankAccount",
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
  "balance": 5250.00
}

# Response: 200 OK
{
  "success": true,
  "data": {
    "id": "wallet-uuid-1",
    "user_id": "user123",
    "name": "Primary Checking",
    "balance": 5250.00,
    "wallet_type": "BankAccount",
    "created_at": "2026-01-28T10:00:00Z",
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

## Transaction Endpoints (Updated)

### Create Transaction (Now Requires Wallet)
```bash
POST /api/transactions
Content-Type: application/json

{
  "user_id": "user123",
  "wallet_id": "wallet-uuid-1",      # NEW: Required field
  "amount": 50.00,
  "transaction_type": "expense",     # "income" or "expense"
  "category": "groceries",
  "description": "Weekly shopping"
}

# Response: 201 Created
{
  "success": true,
  "data": {
    "id": "txn-uuid-1",
    "user_id": "user123",
    "wallet_id": "wallet-uuid-1",    # NEW
    "amount": 50.00,
    "transaction_type": "expense",
    "category": "groceries",
    "description": "Weekly shopping",
    "created_at": "2026-01-28T12:30:00Z",
    "updated_at": "2026-01-28T12:30:00Z"
  }
}

# Note: Wallet balance automatically updated!
# wallet-uuid-1 balance: 5000.00 - 50.00 = 4950.00
```

### Update Transaction (Can Change Wallet)
```bash
PUT /api/transactions/user123/txn-uuid-1
Content-Type: application/json

{
  "wallet_id": "wallet-uuid-2",     # NEW: Move to different wallet
  "amount": 75.00,
  "category": "dining",
  "description": "Lunch at restaurant"
}

# Response: 200 OK
{
  "success": true,
  "data": {
    "id": "txn-uuid-1",
    "user_id": "user123",
    "wallet_id": "wallet-uuid-2",    # Changed!
    "amount": 75.00,
    "transaction_type": "expense",
    "category": "dining",
    "description": "Lunch at restaurant",
    "created_at": "2026-01-28T12:30:00Z",
    "updated_at": "2026-01-28T12:45:00Z"
  }
}

# Note: Wallet balances updated automatically!
# wallet-uuid-1 balance: 4950.00 + 50.00 = 5000.00  (old amount reversed)
# wallet-uuid-2 balance: 200.00 - 75.00 = 125.00    (new amount applied)
```

### Delete Transaction
```bash
DELETE /api/transactions/user123/txn-uuid-1

# Response: 204 No Content

# Note: Wallet balance automatically updated!
# wallet-uuid-2 balance: 125.00 + 75.00 = 200.00 (amount reversed)
```

## Wallet Types

Available wallet types:
- **Cash** - Physical cash wallet
- **BankAccount** - Bank savings/checking account
- **CreditCard** - Credit card
- **Other** - Any other type of account

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
