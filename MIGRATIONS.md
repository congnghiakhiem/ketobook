# SQLx Migrations - Database Setup Guide (2026)

This guide explains how to set up your Supabase database using SQLx migrations for the KetoBook project. This consolidated migration fixes all dependency ordering issues and updates all timestamps to 2026 UTC.

## 🎯 Quick Start: Single Consolidated Migration

The project now uses **one consolidated migration file** that creates all tables in the correct dependency order:

```bash
sqlx migrate run
```

**That's it!** No more "relation 'wallets' does not exist" errors.

### What's Included:

**File:** `migrations/20260101_create_all_tables.sql`

This consolidated migration includes:
1. ✅ `wallet_type` ENUM type (created FIRST, no dependencies)
2. ✅ `wallets` table with credit_limit and all credit card fields
3. ✅ `transactions` table linked to wallets via foreign key (CASCADE DELETE)
4. ✅ `debts` table for debt tracking
5. ✅ Aggregation views for statistics
6. ✅ All timestamps set to `TIMESTAMP WITH TIME ZONE` (UTC, 2026)
7. ✅ All monetary fields use `DECIMAL(15, 2)` for financial precision
8. ✅ All necessary indexes for query performance
9. ✅ Auto-updating timestamp triggers
10. ✅ Comments on all columns explaining semantics

## Schema Overview

### Core Tables (in creation order)

1. **wallets** - User's wallet accounts
   - `id`: UUID primary key (auto-generated)
   - `user_id`: Reference to user
   - `name`: Wallet name (e.g., "Cash", "Checking Account")
   - `balance`: DECIMAL(15, 2) - Current wallet balance
   - `credit_limit`: DECIMAL(15, 2) - Credit card limit (0 for non-credit wallets)
   - `wallet_type`: ENUM - Cash, BankAccount, CreditCard, Other
   - `created_at`, `updated_at`: TIMESTAMP WITH TIME ZONE (auto-managed)

2. **transactions** - Financial transactions
   - `id`: UUID primary key (auto-generated)
   - `user_id`: Reference to user
   - `wallet_id`: Foreign key to wallets (CASCADE DELETE when wallet deleted)
   - `amount`: DECIMAL(15, 2) - Transaction amount (positive values only)
   - `transaction_type`: VARCHAR - "income" or "expense"
   - `category`: VARCHAR - Transaction category
   - `description`: TEXT - Transaction details (nullable)
   - `created_at`, `updated_at`: TIMESTAMP WITH TIME ZONE (auto-managed)

3. **debts** - Debt tracking
   - `id`: UUID primary key (auto-generated)
   - `user_id`: Reference to user
   - `creditor_name`: VARCHAR - Name of creditor
   - `amount`: DECIMAL(15, 2) - Debt amount
   - `interest_rate`: DECIMAL(5, 2) - Annual interest rate
   - `due_date`: TIMESTAMP WITH TIME ZONE - Payment due date (nullable)
   - `status`: VARCHAR - "active", "paid", or "cancelled"
   - `created_at`, `updated_at`: TIMESTAMP WITH TIME ZONE (auto-managed)

### Aggregation Views

- **v_transaction_summary**: Aggregate transaction statistics per user and wallet
- **v_debt_summary**: Aggregate debt statistics per user and status
- **v_wallet_summary**: Wallet statistics with calculated available_credit

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

You should see output like:

```
Applied 20260101_create_all_tables (3.456s)
```

All 7 migration steps execute automatically in correct order:
1. wallet_type ENUM created
2. wallets table created (depends on ENUM)
3. transactions table created (depends on wallets FK)
4. debts table created (independent)
5. All views created
6. All indexes created
7. All triggers created

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

# List types (ENUM)
\dT+

# Should show wallet_type ENUM with values: Cash, BankAccount, CreditCard, Other

# View triggers
\dy

# View indexes
\di

# View created views
\dv

# Should show: v_transaction_summary, v_debt_summary, v_wallet_summary
```

### 5. Wallet-Transaction Relationships

After migration, the schema enforces:

- **Wallet Deletion Cascade**: When a wallet is deleted, all associated transactions are automatically deleted
- **Balance Constraints**: 
  - Wallet balance >= 0
  - Credit limit >= 0
- **Wallet Type Enum**: Enforces wallet types: Cash, BankAccount, CreditCard, Other
- **Transaction Type Validation**: Enforces transaction types: income, expense
- **Financial Precision**: All amounts use DECIMAL(15, 2) for exact calculations
- **Atomic Timestamps**: All created_at/updated_at use UTC timezone

### 6. API Integration with Rust Models

When using the KetoBook API with the updated Rust structs:

#### Transaction Structure (Updated)
```rust
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub wallet_id: String,  // Now required (not Option)
    pub amount: BigDecimal,  // Use BigDecimal, not f64
    pub transaction_type: String,  // "income" or "expense"
    pub category: String,
    pub description: Option<String>,  // Now optional
    pub created_at: DateTime<Utc>,  // Use DateTime<Utc>
    pub updated_at: DateTime<Utc>,  // Use DateTime<Utc>
}
```

#### Debt Structure (Updated)
```rust
pub struct Debt {
    pub id: String,
    pub user_id: String,
    pub creditor_name: String,
    pub amount: BigDecimal,  // Use BigDecimal, not f64 (IMPORTANT!)
    pub interest_rate: BigDecimal,  // Use BigDecimal, not f64 (IMPORTANT!)
    pub due_date: Option<DateTime<Utc>>,  // Now optional
    pub status: String,  // "active", "paid", or "cancelled"
    pub created_at: DateTime<Utc>,  // Use DateTime<Utc>
    pub updated_at: DateTime<Utc>,  // Use DateTime<Utc>
}
```

#### Create a Wallet
```bash
POST /api/wallets
{
  "user_id": "user123",
  "name": "My Checking Account",
  "wallet_type": "BankAccount",
  "balance": "1000.50",
  "credit_limit": null
}
```

#### Create a Transaction (with wallet)
```bash
POST /api/transactions
{
  "user_id": "user123",
  "wallet_id": "wallet-uuid",
  "amount": "50.00",
  "transaction_type": "expense",
  "category": "groceries",
  "description": "Weekly groceries"
}
```

When a transaction is created:
1. The transaction is inserted with the wallet_id (now required)
2. The wallet balance is automatically updated based on transaction type:
   - "income": balance += amount
   - "expense": balance -= amount
3. Cache is invalidated for the user's wallets and transactions

#### Create a Debt
```bash
POST /api/debts
{
  "user_id": "user123",
  "creditor_name": "Bank Loan",
  "amount": "5000.00",
  "interest_rate": "4.50",
  "due_date": "2026-12-31T23:59:59Z"
}
```

All monetary fields in requests and responses now use string representations of BigDecimal for precision.

## Common Issues & Solutions

### Issue: "relation 'wallets' does not exist"

**Root Cause**: Old migration files were in wrong order (SQLx applies them alphabetically)

**✅ FIXED**: Now using single consolidated migration file `20260101_create_all_tables.sql` that handles all dependencies correctly.

**If you previously got this error**:
```bash
# Revert all old migrations (WARNING: DELETES ALL DATA!)
sqlx migrate revert --all

# Clean up old migration files (they're no longer needed)
rm migrations/20250101_create_wallets_table.sql
rm migrations/20250102_create_transactions_table.sql
rm migrations/20250103_add_wallet_id_to_transactions.sql
rm migrations/20250104_add_credit_limit_to_wallets.sql
rm migrations/20250105_create_debts_table.sql
rm migrations/20250106_create_views.sql

# Now run the new consolidated migration
sqlx migrate run
```

### "permission denied for schema public"

**Solution**: Use a Supabase role with proper permissions:
```bash
# Connect with postgres role
DATABASE_URL="postgresql://postgres:<password>@<ref>.supabase.co:5432/postgres" sqlx migrate run
```

### "database does not exist"

**Solution**: Create the database first or use the default `postgres` database from Supabase.

### "Connection timeout"

**Solution**: 
1. Ensure your Supabase project is active (not paused)
2. Check your internet connection
3. Verify the DATABASE_URL is correct
4. Add your IP to Supabase firewall if needed

### "relation already exists"

**Solution**: The migration has already been run successfully.

If you need to start fresh:
```bash
# Revert all migrations (CAUTION: deletes data)
sqlx migrate revert --all

# Then run again
sqlx migrate run
```

### Type mismatches in Rust code

If you see errors like "expected BigDecimal, found f64" or "expected DateTime<Utc>, found String":

**Solution**: Update your Rust structs to match the database schema:
- Use `BigDecimal` for monetary amounts (not `f64`)
- Use `DateTime<Utc>` for timestamps (import via `sqlx::types::chrono`)
- Use `Option<T>` for nullable database columns
- Always derive `#[sqlx::FromRow]`

Example:
```rust
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use rust_decimal::prelude::*;

#[derive(sqlx::FromRow)]
pub struct Transaction {
    pub amount: BigDecimal,  // Not f64!
    pub created_at: DateTime<Utc>,  // Not String!
}
```

## Database Reset

To completely reset the database:

```bash
# Revert all migrations in reverse order
sqlx migrate revert --all

# Confirm tables are deleted
psql "your-database-url" -c "\dt"

# The output should be empty (no tables)

# Now run migrations again
sqlx migrate run
```

**Warning**: This will permanently delete all data. Use with caution in production!

## Manual Migration (Alternative)

If SQLx CLI is not available, you can manually run SQL:

```bash
# Connect to Supabase
psql "your-database-url"

# Load the consolidated migration file
\i migrations/20260101_create_all_tables.sql
```

## Cargo Integration

To verify migrations are ready before building:

```bash
# Check that migrations are prepared
sqlx migrate add -r <name>

# Or prepare existing migrations
cargo sqlx prepare
```

## Next Steps

1. ✅ Install SQLx CLI: `cargo install sqlx-cli --no-default-features --features postgres`
2. ✅ Configure `.env` with DATABASE_URL pointing to Supabase
3. ✅ Run migrations: `sqlx migrate run`
4. ✅ Verify tables: `psql "your-database-url" -c "\dt"`
5. ✅ Build project: `cargo build`
6. ✅ Run tests: `cargo test`
7. ✅ Start API: `cargo run`

## References

- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [SQLx Migrations](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
- [Supabase Database](https://supabase.com/docs/guides/database)
- [PostgreSQL DECIMAL Type](https://www.postgresql.org/docs/current/datatype-numeric.html)
- [Chrono DateTime](https://docs.rs/chrono/latest/chrono/)
- [Rust Decimal](https://docs.rs/rust_decimal/latest/rust_decimal/)

---

**Migration Status**: ✅ Ready to deploy! 🚀

All tables will be created correctly with proper dependency ordering in a single consolidated migration.
