# KetoBook Finance Management API

A modern, scalable finance management API built with **Actix-web**, **Supabase PostgreSQL**, and **Upstash Redis**. Featuring **multi-wallet support**, **credit card management**, **atomic transactions**, and **financial precision** with BigDecimal. Implements cache-aside pattern for optimal performance while managing personal transactions, multiple wallets, and debts.

## 🏗️ Architecture

### Project Structure
```
src/
├── main.rs           # Server setup and route registration
├── config.rs         # Environment configuration (dotenv)
├── models.rs         # Data models with BigDecimal financial precision
├── db.rs             # PostgreSQL connection pool management
├── cache.rs          # Redis connection and cache-aside pattern
├── wallets.rs        # Wallet CRUD endpoints (NEW)
├── transactions.rs   # Transaction CRUD with atomic operations (ENHANCED)
└── debts.rs          # Debt CRUD endpoints
```

### Key Features

- **Multi-Wallet Support**: Manage multiple wallets (Cash, Bank Account, Credit Card, Other)
- **Credit Card Management**: Track credit cards with credit limits and available credit
- **Atomic Transactions**: All transaction operations use PostgreSQL atomic transactions (BEGIN/COMMIT/ROLLBACK)
- **Financial Precision**: BigDecimal for monetary values (no floating-point errors)
- **Smart Balance Validation**: Different validation logic for credit cards vs regular wallets
- **Automatic Balance Updates**: Wallet balances automatically updated when transactions change
- **Async/Await**: Built with Tokio runtime for high concurrency
- **Cache-Aside Pattern**: Redis integration with wallet-specific invalidation
- **Type-Safe Database Access**: SQLx for compile-time SQL verification
- **RESTful API**: Clean endpoint design with standardized responses
- **Modular Design**: Separated concerns with independent modules
- **Supabase PostgreSQL**: Serverless PostgreSQL with built-in auth and real-time
- **Upstash Redis**: Serverless Redis for edge deployment

## 📋 Prerequisites

- **Rust** 1.91.1 (specified in `rust-toolchain.toml`, install from [rustup.rs](https://rustup.rs))
- **Supabase PostgreSQL** (free tier at [supabase.com](https://supabase.com)) or local PostgreSQL
- **Upstash Redis** (free tier available at [upstash.com](https://upstash.com)) or local Redis

### Quick Start with Docker (Local Development)


```bash
# Start PostgreSQL
docker run --name ketobook-db -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres:15

# Start Redis
docker run --name ketobook-redis -p 6379:6379 -d redis:7-alpine
```

### Production Setup with Upstash

1. Create account at [upstash.com](https://upstash.com)
2. Create a new Redis database (free tier available)
3. Copy the connection URL
4. Set `REDIS_URL` in your `.env` file

## 🚀 Getting Started

### 1. Clone and Setup Environment
```bash
cd ketobook
cp .env.example .env
```

### 2. Configure .env
Edit `.env` with your Supabase and Redis credentials:

**For local development:**
```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/ketobook_db
REDIS_URL=redis://127.0.0.1:6379
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=info
```

**For production with Supabase + Upstash:**
```env
DATABASE_URL=postgresql://postgres:<your-password>@<ref>.supabase.co:5432/postgres
REDIS_URL=redis://default:<auth_token>@<host>:<port>
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info
```

Get your credentials from:
- **Supabase**: Project Settings → Database → Connection string (PostgreSQL)
- **Upstash**: Dashboard → Redis → Connection

### 3. Initialize Database Schema
```bash
# If using local PostgreSQL, create database
createdb ketobook_db

# Apply schema (run in your database client)
psql "your-database-url" < schema.sql
```
```

### 4. Build and Run
```bash
# Debug build (faster compilation)
cargo run

# Release build (optimized)
cargo build --release
./target/release/ketobook
```

Server will start at `http://127.0.0.1:8080`

## 📚 API Endpoints

### Health Check
```bash
GET /health
```

### Wallets (NEW)

#### Get all wallets for a user
```bash
GET /api/wallets/user/{user_id}
```

#### Get single wallet
```bash
GET /api/wallets/{user_id}/{wallet_id}
```

#### Create wallet (all types)
```bash
POST /api/wallets
Content-Type: application/json

{
  "user_id": "user_123",
  "name": "My Credit Card",
  "wallet_type": "CreditCard",
  "balance": "2500.00",
  "credit_limit": "10000.00"
}
```

#### Update wallet
```bash
PUT /api/wallets/{user_id}/{wallet_id}
Content-Type: application/json

{
  "name": "Primary Credit Card",
  "balance": "2750.00",
  "credit_limit": "15000.00"
}
```

#### Delete wallet
```bash
DELETE /api/wallets/{user_id}/{wallet_id}
```

### Transactions (ENHANCED with Atomic Operations)

#### Get all transactions for a user
```bash
GET /api/transactions/user/{user_id}
```

#### Get single transaction
```bash
GET /api/transactions/{user_id}/{transaction_id}
```

#### Create transaction (now with wallet requirement and balance validation)
```bash
POST /api/transactions
Content-Type: application/json

{
  "user_id": "user_123",
  "wallet_id": "wallet_uuid",
  "amount": "50.00",
  "transaction_type": "expense",
  "category": "groceries",
  "description": "Weekly grocery shopping"
}

# Atomic operation: Transaction created + wallet balance updated in single unit
# For credit cards: validates available_credit >= amount
# For regular wallets: validates balance >= amount (for expenses)
```

#### Update transaction (can change wallet and amount)
```bash
PUT /api/transactions/{user_id}/{transaction_id}
Content-Type: application/json

{
  "wallet_id": "different_wallet_id",
  "amount": "75.00",
  "category": "food",
  "description": "Updated description"
}

# Atomic: Old wallet balance reversed + new wallet balance applied
```

#### Delete transaction
```bash
DELETE /api/transactions/{user_id}/{transaction_id}

# Atomic: Transaction deleted + balance reversal in single unit
```

### Debts

#### Get all debts for a user
```bash
GET /api/debts/user/{user_id}
```

#### Get single debt
```bash
GET /api/debts/{user_id}/{debt_id}
```

#### Create debt
```bash
POST /api/debts
Content-Type: application/json

{
  "user_id": "user_123",
  "creditor_name": "Credit Card Co",
  "amount": 5000.00,
  "interest_rate": 18.5,
  "due_date": "2025-12-31T23:59:59Z"
}
```

#### Update debt
```bash
PUT /api/debts/{user_id}/{debt_id}
Content-Type: application/json

{
  "amount": 4500.00,
  "status": "active"
}
```

#### Delete debt
```bash
DELETE /api/debts/{user_id}/{debt_id}
```

## 🔄 Cache-Aside Pattern

The application implements the cache-aside pattern across all read operations:

1. **Request arrives** → Check Redis cache
2. **Cache hit** → Return cached data (fast path)
3. **Cache miss** → Query PostgreSQL
4. **Store in cache** → Set with 1-hour TTL
5. **Return data** → Client receives fresh data

**Write operations** invalidate relevant cache keys to maintain data consistency:
- Create/Update/Delete operations clear the cache
- **Wallet-specific invalidation**: Updates to one wallet only clear that wallet's cache
- Pattern-based invalidation ensures all related caches are cleared

## ⚛️ Atomic Transactions

All transaction operations (create, update, delete) use PostgreSQL atomic transactions:

- **BEGIN/COMMIT/ROLLBACK**: All changes succeed or fail together
- **No partial updates**: Transaction creation and balance update are atomic
- **Automatic rollback**: Any error rolls back all changes
- **Balance consistency**: Wallet balance always matches transaction history

**Example atomicity scenario:**
```
1. Customer creates $500 expense on credit card
2. System begins database transaction
3. Validates: available_credit ($7500) >= $500 ✓
4. Inserts transaction record
5. Updates wallet: balance = $2500 + $500 = $3000 (more debt)
6. Commits both changes atomically
   → If commit fails: BOTH changes rolled back
   → Wallet balance unchanged, transaction never created
```

## 💰 Financial Precision

All monetary values use **BigDecimal** instead of floating-point:

- **Accuracy**: Exact to 2 decimal places (cents)
- **No rounding errors**: 0.1 + 0.2 = 0.3 exactly
- **Safe arithmetic**: All financial calculations precise
- **Database alignment**: Matches DECIMAL(15, 2) columns
- **JSON serialization**: Represented as strings for precision

**Example:**
```json
{
  "balance": "5000.50",
  "credit_limit": "10000.00",
  "amount": "75.25"
}
```

## 🏦 Multi-Wallet Types

Supported wallet types with different balance semantics:

| Type | Balance Meaning | Validation | Use Case |
|------|-----------------|-----------|----------|
| **Cash** | Amount of cash | Cannot go negative | Physical cash |
| **BankAccount** | Account balance | Cannot go negative | Checking/savings |
| **CreditCard** | Current debt | Available credit ≥ amount | Credit cards |
| **Other** | Custom balance | Cannot go negative | Other accounts |

**Credit Card Specifics:**
- `balance` field = current debt (0 = no debt, limit = maxed out)
- `credit_limit` field = spending limit
- `available_credit` = limit - balance (calculated property)
- Expense transactions validated: `amount <= available_credit`
- Income transactions reduce debt: `balance -= amount`

## 🗄️ Database Schema

The application uses PostgreSQL with the following tables:

```sql
-- Wallets table (NEW)
CREATE TABLE wallets (
  id VARCHAR PRIMARY KEY,
  user_id VARCHAR NOT NULL,
  name VARCHAR(100) NOT NULL,
  balance DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
  credit_limit DECIMAL(15, 2) DEFAULT 0.00,
  wallet_type VARCHAR NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

-- Transactions table (UPDATED)
CREATE TABLE transactions (
  id VARCHAR PRIMARY KEY,
  user_id VARCHAR NOT NULL,
  wallet_id VARCHAR NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
  amount DECIMAL(15, 2) NOT NULL,
  transaction_type VARCHAR NOT NULL,
  category VARCHAR NOT NULL,
  description VARCHAR,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

-- Debts table
CREATE TABLE debts (
  id VARCHAR PRIMARY KEY,
  user_id VARCHAR NOT NULL,
  creditor_name VARCHAR NOT NULL,
  amount DECIMAL(15, 2) NOT NULL,
  interest_rate DECIMAL(5, 2) NOT NULL,
  due_date TIMESTAMP NOT NULL,
  status VARCHAR NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_wallets_user_id ON wallets(user_id);
CREATE INDEX idx_wallets_credit_card ON wallets(user_id, wallet_type) WHERE wallet_type = 'CreditCard';
CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_wallet_id ON transactions(wallet_id);
CREATE INDEX idx_debts_user_id ON debts(user_id);
```

## 📦 Dependencies

Key dependencies in `Cargo.toml`:

- **actix-web** (4.0) - Web framework
- **tokio** - Async runtime
- **sqlx** (0.7) - Database driver with async support and BigDecimal feature
- **bigdecimal** (0.3) - Financial precision with serde support
- **redis** (0.25) - Redis client with async support
- **serde** - JSON serialization/deserialization
- **chrono** - Date/time handling
- **uuid** - ID generation
- **dotenv** - Environment configuration

## 🔍 Development

### Run with Debug Logging
```bash
RUST_LOG=debug cargo run
```

### Check for Compilation Issues
```bash
cargo check
```

### Format Code
```bash
cargo fmt
```

### Lint Code
```bash
cargo clippy
```

## 🚨 Error Handling

All endpoints return standardized JSON responses:

### Success Response
```json
{
  "success": true,
  "data": { /* entity data */ },
  "error": null
}
```

### Error Response
```json
{
  "success": false,
  "data": null,
  "error": "Descriptive error message"
}
```

## 🔐 Security Considerations

- Use strong database passwords
- Keep Redis access restricted to localhost or VPN
- Validate user_id authorization in production
- Use HTTPS in production
- Implement rate limiting (not included in scaffold)
- Add user authentication layer (JWT, OAuth)

## 📈 Performance Tips

1. **Database Indexing**: Ensure indexes on `user_id` for fast queries
2. **Connection Pooling**: Currently set to 5 connections, adjust as needed
3. **Cache TTL**: 1-hour TTL can be customized in `cache.rs`
4. **Batch Operations**: Combine multiple requests when possible
5. **Query Optimization**: Use EXPLAIN ANALYZE for slow queries

## 🛠️ Troubleshooting

### Database Connection Errors
```bash
# Verify PostgreSQL is running
psql -U postgres -h localhost

# Check DATABASE_URL format
# postgresql://user:password@host:port/dbname
```

### Redis Connection Errors
```bash
# Verify Redis is running
redis-cli ping  # Should return "PONG"

# Check REDIS_URL format
# redis://127.0.0.1:6379
```

### Schema Not Found
```bash
# Verify tables exist
psql ketobook_db -c "\dt"

# Run schema.sql if tables are missing
psql ketobook_db < schema.sql
```

## 📝 Next Steps

1. **Authentication**: Add JWT-based authentication
2. **Authorization**: Implement user isolation
3. **Validation**: Add comprehensive input validation
4. **Testing**: Write unit and integration tests
5. **Pagination**: Add limit/offset to list endpoints
6. **Filtering**: Allow filtering by date range, category, status
7. **Aggregations**: Add summary endpoints (total income, total debt, etc.)
8. **API Documentation**: Generate OpenAPI/Swagger docs

## 📄 License

This project is licensed under the MIT License. See LICENSE file for details.

## 👨‍💻 Architecture Notes

**Why Actix-web?**
- Extremely fast (benchmarks show superior performance)
- Excellent async/await support with Tokio
- Great middleware ecosystem
- Type-safe routing

**Why SQLx?**
- Compile-time query verification
- No runtime string parsing
- Better error messages
- Excellent PostgreSQL support
- BigDecimal support for financial precision

**Why BigDecimal?**
- Accurate financial calculations (no 0.1 + 0.2 = 0.30000000000000004)
- Exact to 2 decimal places (cents)
- Matches database DECIMAL(15, 2) type
- Essential for money operations

**Why Atomic Transactions?**
- Guarantees consistency: Transaction + balance update always together
- Prevents partial updates: Either both succeed or both fail
- Automatic rollback: Error rolls back all changes
- Critical for financial systems

**Why Redis + Postgres?**
- Hot data in Redis for sub-millisecond access
- Persistent data in PostgreSQL for durability
- Easy invalidation strategy with cache-aside pattern
- Wallet-specific caching for multi-wallet optimization

## 🚀 Quick Links

- **API Reference**: See [API_WALLET_REFERENCE.md](API_WALLET_REFERENCE.md) for detailed endpoint documentation
- **Setup Guide**: See [SETUP.md](SETUP.md) for installation instructions
- **Implementation Details**: See [WALLET_REFACTOR_SUMMARY.md](WALLET_REFACTOR_SUMMARY.md) for technical architecture
- **Checklist**: See [IMPLEMENTATION_CHECKLIST.md](IMPLEMENTATION_CHECKLIST.md) for feature status

---

Built with ❤️ by Senior Rust Backend Engineer