# SQLx Migrations - Database Setup Guide

This guide explains how to set up your Supabase database using SQLx migrations for the KetoBook project.

## Database Schema Overview

### Tables

1. **wallets** - User's wallet accounts
   - `id`: UUID primary key
   - `user_id`: Reference to user
   - `name`: Wallet name (e.g., "Cash", "Checking Account")
   - `balance`: Current wallet balance
   - `wallet_type`: Enum (Cash, BankAccount, CreditCard, Other)
   - `created_at`, `updated_at`: Timestamps

2. **transactions** - Financial transactions
   - `id`: UUID primary key
   - `user_id`: Reference to user
   - `wallet_id`: Foreign key to wallets (updated/deleted with wallet)
   - `amount`: Transaction amount (positive values only)
   - `transaction_type`: "income" or "expense"
   - `category`: Transaction category
   - `description`: Transaction details
   - `created_at`, `updated_at`: Timestamps

3. **debts** - Debt tracking
   - `id`: UUID primary key
   - `user_id`: Reference to user
   - `creditor_name`: Name of creditor
   - `amount`: Debt amount
   - `interest_rate`: Annual interest rate
   - `due_date`: Payment due date
   - `status`: "active" or "paid"
   - `created_at`, `updated_at`: Timestamps

### Views

- **v_transaction_summary**: Aggregate transaction statistics per user
- **v_debt_summary**: Aggregate debt statistics per user

## Prerequisites

Before running migrations, ensure you have:
1. SQLx CLI installed
2. A Supabase account with a project created
3. Your DATABASE_URL configured in `.env`

## Installation

### Install SQLx CLI

```bash
# Install SQLx CLI with PostgreSQL support
cargo install sqlx-cli --no-default-features --features postgres

# Verify installation
sqlx --version
```

## Setup Steps

### 1. Configure Environment

Make sure your `.env` file has the correct DATABASE_URL:

```env
DATABASE_URL=postgresql://postgres:<password>@<ref>.supabase.co:5432/postgres
```

To get your Supabase connection string:
1. Go to https://supabase.com and sign in
2. Select your project
3. Click "Settings" → "Database" → "Connection Pooling" or "Direct Connection"
4. Copy the PostgreSQL URI
5. Replace `[YOUR-PASSWORD]` with your actual password

### 2. Create Database (if needed)

Supabase automatically creates a `postgres` database, but you can create a dedicated one:

```bash
# Using psql
psql "postgresql://postgres:<password>@<ref>.supabase.co:5432/postgres" -c "CREATE DATABASE ketobook_db;"

# Or create via Supabase dashboard
# SQL Editor → New Query → paste the CREATE DATABASE statement
```

### 3. Run Migrations

Navigate to the project root and run:

```bash
# Run all pending migrations
sqlx migrate run

# Or with specific database URL
sqlx migrate run --database-url "postgresql://postgres:<password>@<ref>.supabase.co:5432/postgres"
```

The migrations should be applied in dependency order:
1. `20250101_create_wallets_table.sql` - Creates wallets table (MUST RUN FIRST)
2. `20250102_create_transactions_table.sql` - Creates transactions table
3. `20250103_add_wallet_id_to_transactions.sql` - Adds wallet_id FK to transactions
4. `20250104_add_credit_limit_to_wallets.sql` - Adds credit_limit for credit cards
5. `20250105_create_debts_table.sql` - Creates debts table
6. `20250106_create_views.sql` - Creates aggregation views

✅ **Files have been renamed with sequential dates (20250101-20250106) to ensure correct execution order**

You should see output like:

```
Applied 20250101_create_wallets_table (2.123s)
Applied 20250102_create_transactions_table (1.456s)
Applied 20250103_add_wallet_id_to_transactions (1.789s)
Applied 20250104_add_credit_limit_to_wallets (0.956s)
Applied 20250105_create_debts_table (1.234s)
Applied 20250106_create_views (0.789s)
```

#### Expected Output

```
Applied 20250128_create_wallets_table (2.123s)
Applied 20250128_create_transactions_table (1.456s)
Applied 20250128_add_wallet_id_to_transactions (1.789s)
Applied 20250128_add_credit_limit_to_wallets (0.956s)
Applied 20250128_create_debts_table (1.234s)
Applied 20250128_create_views (0.789s)
```

### 4. Verify Migrations

Check that tables were created:

```bash
# Connect to your database
psql "your-database-url"

# List tables
\dt

# Expected output should show:
# - public | wallets
# - public | transactions
# - public | debts
```

### 5. Wallet-Transaction Relationships

After migration, the schema enforces:

- **Wallet Deletion Cascade**: When a wallet is deleted, all associated transactions are also deleted
- **Balance Constraints**: Wallet balance cannot go negative (except for credit cards with negative balances)
- **Wallet Type Enum**: Enforces wallet types to be Cash, BankAccount, CreditCard, or Other
- **Foreign Key Constraints**: Transaction wallet_id must reference an existing wallet

### 6. API Integration

When using the KetoBook API:

#### Create a Wallet
```bash
POST /api/wallets
{
  "user_id": "user123",
  "name": "My Checking Account",
  "wallet_type": "BankAccount",
  "balance": 1000.50
}
```

#### Create a Transaction (with wallet)
```bash
POST /api/transactions
{
  "user_id": "user123",
  "wallet_id": "wallet-uuid",
  "amount": 50.00,
  "transaction_type": "expense",
  "category": "groceries",
  "description": "Weekly groceries"
}
```

When a transaction is created:
1. The transaction is inserted with the wallet_id
2. The wallet balance is automatically updated based on transaction type:
   - "income": balance += amount
   - "expense": balance -= amount
3. Cache is invalidated for the user's wallets and transactions

#### Update a Transaction
```bash
PUT /api/transactions/{user_id}/{transaction_id}
{
  "wallet_id": "different-wallet-id",  // Optional: move to different wallet
  "amount": 75.00,                       // Optional: update amount
  "category": "dining"                   // Optional: update category
}
```

When updated:
1. If wallet_id changes, old wallet balance is reversed, new wallet balance is adjusted
2. If amount changes, the balance delta is applied to the wallet
3. All relevant caches are invalidated

# View triggers
\dy

# View indexes
\di

# View created views
\dv
```

## Migration Files

The project includes six migrations (renamed with sequential dates for correct execution order):

### 1. `20250101_create_wallets_table.sql` ⭐ MUST RUN FIRST
- Creates `wallets` table with multi-wallet support
- Defines `wallet_type` ENUM (Cash, BankAccount, CreditCard, Other)
- Adds `balance` DECIMAL(15, 2) - wallet balance
- Adds `credit_limit` DECIMAL(15, 2) - credit card limit (0.00 for non-credit wallets)
- Adds indexes on `user_id`, `wallet_type`
- Adds auto-update trigger for `updated_at`
- **Financial Precision**: Uses DECIMAL(15, 2) for accurate monetary values

### 2. `20250102_create_transactions_table.sql`
- Creates `transactions` table
- Adds `amount` as DECIMAL(15, 2) for financial precision
- Adds `transaction_type` VARCHAR (must be "income" or "expense")
- Adds indexes on `user_id`, `created_at`, `wallet_id` (composite)
- Adds auto-update trigger for `updated_at`

### 3. `20250103_add_wallet_id_to_transactions.sql`
- Adds `wallet_id` column to transactions table (links to wallets)
- Creates foreign key constraint with CASCADE DELETE
- Adds composite indexes for efficient wallet-based queries
- **Dependency**: Requires wallets table to exist

### 4. `20250104_add_credit_limit_to_wallets.sql` ⭐ NEW - CREDIT CARD SUPPORT
- Adds `credit_limit` field to wallets table
- Adds check constraint: `credit_limit >= 0`
- Creates specialized index `idx_wallets_credit_card` for credit card filtering
- Enables credit card balance semantics:
  - `balance` = current debt (0 = no debt, limit = fully used)
  - `available_credit` = limit - balance
  - Used for transaction validation: `amount <= available_credit`
- **Dependency**: Requires wallets table to exist

### 5. `20250105_create_debts_table.sql`
- Creates `debts` table
- Adds `amount` as DECIMAL(15, 2) for financial precision
- Adds indexes on `user_id`, `status`, `due_date`
- Adds auto-update trigger for `updated_at`

### 6. `20250106_create_views.sql`
- Creates `v_transaction_summary` view for aggregate statistics
- Creates `v_debt_summary` view for debt statistics
- Creates `v_wallet_summary` view for wallet statistics

## ⚠️ COMMON ERROR: "relation 'wallets' does not exist"

**✅ FIXED**: Files have been renamed with sequential dates (20250101-20250106) to ensure correct execution order!

### Migration Execution Order is Now Correct:

```
sqlx migrate run
```

Will now automatically run in correct order:
1. 20250101_create_wallets_table.sql
2. 20250102_create_transactions_table.sql
3. 20250103_add_wallet_id_to_transactions.sql
4. 20250104_add_credit_limit_to_wallets.sql
5. 20250105_create_debts_table.sql
6. 20250106_create_views.sql

### If You Already Applied Migrations:

If you applied the migrations with the old filenames and got the error, you need to revert and reapply:

```bash
# Revert all migrations (WARNING: DELETES ALL DATA!)
sqlx migrate revert --all

# Now run with corrected order
sqlx migrate run
```

## Troubleshooting

### "permission denied for schema public"

**Solution**: Use a Supabase role with proper permissions:
```bash
# Connect with postgres role
DATABASE_URL="postgresql://postgres:<password>@<ref>.supabase.co:5432/postgres" sqlx migrate run
```

### "database does not exist"

**Solution**: Create the database first or use the default `postgres` database from Supabase.

### Connection timeout

**Solution**: 
1. Ensure your Supabase project is active (not paused)
2. Check your internet connection
3. Verify the DATABASE_URL is correct
4. Add your IP to Supabase firewall if needed

### "relation already exists"

**Solution**: The migration has already been run. To reset:
```bash
# Revert all migrations (CAUTION: deletes data)
sqlx migrate revert --all

# Then run again
sqlx migrate run
```

## Rolling Back Migrations

To remove all migrations and start fresh:

```bash
# Revert all migrations in reverse order
sqlx migrate revert --all

# Confirm tables are deleted
psql "your-database-url" -c "\dt"
```

**Warning**: This will permanently delete all data in the tables.

## Manual Migration (Alternative)

If SQLx CLI is not available, you can manually run SQL:

```bash
# Connect to Supabase
psql "your-database-url"

# Copy and paste contents of migration files
# Or load from file
\i migrations/20250128_create_transactions_table.sql
\i migrations/20250128_create_debts_table.sql
\i migrations/20250128_create_views.sql
```

## Integrating with Cargo Build

To automatically run migrations on `cargo build`, add to `src/main.rs`:

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ... existing code ...
    
    // Run migrations
    sqlx::migrate!()
        .run(db_pool.get_pool())
        .await
        .expect("Failed to run migrations");
    
    // ... rest of main ...
}
```

However, SQLx CLI must be run manually first to prepare compiled migrations.

## Next Steps

1. Run migrations with SQLx CLI
2. Verify tables exist in Supabase
3. Run `cargo run` to start the API
4. Test endpoints with `test_api.sh` or `test_api.ps1`

## References

- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [SQLx Migrations](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
- [Supabase Database](https://supabase.com/docs/guides/database)

---

**Status**: Ready to migrate! 🚀
