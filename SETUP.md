# KetoBook API - Complete Setup Guide

This guide walks you through setting up the KetoBook Finance Management API from scratch.

## 1. Prerequisites Installation

### Rust Installation (Version 1.91.1 Required)
```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Rust 1.91.1 (required for this project)
rustup install 1.91.1
rustup default 1.91.1

# Verify installation
rustc --version  # Should show 1.91.1
cargo --version
```

**Note**: The project includes `rust-toolchain.toml` that specifies version 1.91.1. This ensures consistency across all developers.

### PostgreSQL Installation (Using Supabase)

**Option 1: Supabase Cloud (Recommended for Production)**

1. Visit https://supabase.com and sign up for free
2. Create a new project
3. Go to Project Settings → Database → Connection string
4. Copy the PostgreSQL connection string
5. Update your `.env` file:
```env
DATABASE_URL=postgresql://postgres:<password>@<project>.supabase.co:5432/postgres
```

The URL format is: `postgresql://postgres:<your-password>@<ref>.supabase.co:5432/postgres`

**Option 2: Local PostgreSQL (Development)**

**Windows (via Chocolatey):**
```powershell
choco install postgresql --version=15.1
```

**macOS:**
```bash
brew install postgresql@15
brew services start postgresql@15
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Local PostgreSQL URL:**
```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/ketobook_db
```

**Verify Installation:**
```bash
psql --version
psql -U postgres  # Connect and verify
```

### Redis Installation (Using Upstash)

**Option 1: Upstash Cloud (Recommended for Production)**

1. Visit https://upstash.com and create a free account
2. Create a new Redis database
3. Copy the connection URL from your Upstash dashboard
4. Update your `.env` file:
```env
REDIS_URL=redis://default:<password>@<host>:<port>
```

The URL format is: `redis://default:<auth_token>@<host>:<port>`

**Option 2: Local Redis (Development)**

**Windows (via WSL or Docker):**
```bash
# Using Docker (easiest)
docker run --name ketobook-redis -p 6379:6379 -d redis:7-alpine

# Update .env for local Redis
REDIS_URL=redis://127.0.0.1:6379
```

**macOS:**
```bash
brew install redis
brew services start redis
# Redis runs on localhost:6379
REDIS_URL=redis://127.0.0.1:6379
```

**Linux:**
```bash
sudo apt install redis-server
sudo systemctl start redis-server
# Redis runs on localhost:6379
REDIS_URL=redis://127.0.0.1:6379
```

**Verify Installation:**
```bash
redis-cli ping  # Should return "PONG"
```

## 2. Project Setup

### Clone/Create Project
```bash
cd c:\Projects  # or your preferred directory
cd ketobook
```

### Copy Environment Configuration
```bash
cp .env.example .env
```

### Edit .env File
Update with your actual credentials:
```env
DATABASE_URL=postgresql://postgres:your_password@localhost:5432/ketobook_db
REDIS_URL=redis://127.0.0.1:6379
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=info
```

## 3. Database Setup with SQLx Migrations

### Install SQLx CLI

```bash
# Install SQLx CLI (required for running migrations)
cargo install sqlx-cli --no-default-features --features postgres

# Verify installation
sqlx --version
```

### Update .env with Database URL

Edit your `.env` file with Supabase credentials:

```env
# For Supabase
DATABASE_URL=postgresql://postgres:<password>@<ref>.supabase.co:5432/postgres

# For local PostgreSQL
DATABASE_URL=postgresql://postgres:password@localhost:5432/ketobook_db
```

### Run Database Migrations

```bash
# Navigate to project directory
cd c:\Projects\ketobook

# Run all migrations
sqlx migrate run

# You should see:
# Applied 20250128_create_transactions_table
# Applied 20250128_create_debts_table
# Applied 20250128_create_views
```

**For Supabase**, the migrations create:
- `transactions` table for income/expense tracking
- `debts` table for loan management
- Indexes for optimal query performance
- Auto-updating timestamp triggers
- Aggregation views for summaries

### Verify Migrations

Connect to your database and check tables were created:

```bash
# Connect to your database
psql "your-database-url"

# List tables (should show transactions and debts)
\dt

# List views (should show v_transaction_summary and v_debt_summary)
\dv

# List triggers (should show auto-update triggers)
\dy
```

### Alternative: Manual Migration

If SQLx CLI isn't available, manually run the SQL files:

```bash
# For Supabase via SQL Editor
# 1. Go to Supabase Dashboard
# 2. Click "SQL Editor" → "New Query"
# 3. Copy contents from migrations/20250128_create_transactions_table.sql
# 4. Click "Run"
# 5. Repeat for other migration files

# Or via psql
psql "your-database-url" < migrations/20250128_create_transactions_table.sql
psql "your-database-url" < migrations/20250128_create_debts_table.sql
psql "your-database-url" < migrations/20250128_create_views.sql
```

## 4. Previous: Manual Database Setup
```bash
# Using psql
psql -U postgres

# In psql prompt
CREATE DATABASE ketobook_db;
\connect ketobook_db

# Exit psql
\q
```

### Run Database Schema
```bash
# From project root
psql -U postgres -d ketobook_db -f schema.sql
```

**Verify Schema:**
```bash
psql -U postgres -d ketobook_db
\dt  # List all tables

# You should see:
#  - public | transactions
#  - public | debts
```

### Create Test Data (Optional)
```sql
-- Connect to database first
psql -U postgres -d ketobook_db

-- Insert sample transaction
INSERT INTO transactions (id, user_id, amount, transaction_type, category, description, created_at, updated_at)
VALUES (
  'txn_001',
  'user_123',
  150.50,
  'expense',
  'groceries',
  'Weekly grocery shopping',
  NOW(),
  NOW()
);

-- Insert sample debt
INSERT INTO debts (id, user_id, creditor_name, amount, interest_rate, due_date, status, created_at, updated_at)
VALUES (
  'debt_001',
  'user_123',
  'Bank of America',
  5000.00,
  18.5,
  '2025-12-31T23:59:59Z',
  'active',
  NOW(),
  NOW()
);
```

## 4. Build the Project

### Install Dependencies
```bash
# This happens automatically during build, but you can pre-check
cargo check
```

### Build in Debug Mode (Faster)
```bash
cargo build
```

### Build in Release Mode (Optimized)
```bash
cargo build --release
# Output will be in target/release/ketobook
```

## 5. Run the Server

### Start with Cargo
```bash
cargo run
```

### Start Compiled Binary (Release)
```bash
./target/release/ketobook  # macOS/Linux
.\target\release\ketobook.exe  # Windows
```

### Expected Output
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs
     Running `target/debug/ketobook`
[INFO ] Loaded configuration: AppConfig { ... }
[INFO ] Database pool initialized successfully
[INFO ] Redis cache initialized successfully
[INFO ] Starting server on 127.0.0.1:8080
```

## 6. Test the API

### Health Check
```bash
curl http://localhost:8080/health
```

**Expected Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-01-28T10:30:00Z"
}
```

### Create a Transaction
```bash
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_123",
    "amount": 50.00,
    "transaction_type": "expense",
    "category": "food",
    "description": "Lunch"
  }'
```

### Get User Transactions
```bash
curl http://localhost:8080/api/transactions/user/user_123
```

### Create a Debt
```bash
curl -X POST http://localhost:8080/api/debts \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_123",
    "creditor_name": "Credit Card",
    "amount": 1000.00,
    "interest_rate": 18.5,
    "due_date": "2025-12-31T23:59:59Z"
  }'
```

## 7. Development Workflow

### Running in Debug Mode with Logging
```bash
RUST_LOG=debug cargo run
```

### Code Formatting
```bash
# Format all Rust code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Linting
```bash
# Check for code issues
cargo clippy
```

### Testing (when added)
```bash
cargo test
```

## 8. Docker Setup (Alternative)

### Using Docker Compose
Create `docker-compose.yml`:
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_PASSWORD: password
      POSTGRES_DB: ketobook_db
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

volumes:
  postgres_data:
```

**Start Services:**
```bash
docker-compose up -d
```

**Run Application:**
```bash
cargo run
```

## 9. Production Deployment Checklist

- [ ] Set `RUST_LOG=info` (not debug)
- [ ] Use `cargo build --release` for optimized binary
- [ ] Run database migrations/schema setup
- [ ] Configure proper SECRET_KEY and credentials
- [ ] Set up SSL/TLS certificates
- [ ] Configure firewall rules
- [ ] Set up log aggregation
- [ ] Add health check monitoring
- [ ] Configure database backups
- [ ] Test disaster recovery procedures
- [ ] Set up alerts for errors
- [ ] Review security headers
- [ ] Enable CORS properly if needed
- [ ] Add rate limiting
- [ ] Implement request logging

## 10. Troubleshooting

### Port Already in Use
```bash
# Change port in .env
SERVER_PORT=8081
```

### Database Connection Refused
```bash
# Check PostgreSQL is running
psql -U postgres -h localhost  # Test connection

# Verify DATABASE_URL format
# postgresql://user:password@host:port/dbname
```

### Redis Connection Error
```bash
# Check Redis is running
redis-cli ping

# Verify REDIS_URL
# redis://127.0.0.1:6379
```

### Module Not Found Errors
```bash
# Clean and rebuild
cargo clean
cargo build
```

### Compilation Errors with Old Rust Version
```bash
# Update Rust
rustup update
```

## 11. Project Structure Explanation

```
ketobook/
├── src/
│   ├── main.rs           # Application entry point, server setup
│   ├── config.rs         # Environment configuration loading
│   ├── models.rs         # Serde data models for API
│   ├── db.rs             # PostgreSQL connection pool
│   ├── cache.rs          # Redis and cache-aside pattern
│   ├── transactions.rs   # Transaction CRUD routes and handlers
│   └── debts.rs          # Debt CRUD routes and handlers
├── Cargo.toml            # Rust dependencies
├── .env.example          # Environment variables template
├── .env                  # Actual environment variables (git ignored)
├── schema.sql            # PostgreSQL schema definition
└── README.md             # Full documentation
```

## 12. Next Development Steps

1. **Add Authentication**
   - JWT token validation
   - User identity verification
   - Role-based access control

2. **Input Validation**
   - Use `validator` crate
   - Validate amounts, dates, user IDs
   - Return meaningful error messages

3. **API Documentation**
   - Add Swagger/OpenAPI support
   - Use `actix-web-swagger` or similar

4. **Testing**
   - Unit tests for business logic
   - Integration tests for API endpoints
   - Database migration tests

5. **Advanced Features**
   - Pagination for list endpoints
   - Filtering and sorting
   - Aggregation endpoints (summaries, reports)
   - Export to CSV/PDF

6. **Monitoring & Metrics**
   - Prometheus metrics
   - Application performance monitoring
   - Database query logging

## Support

For issues or questions, refer to:
- [Actix-web Documentation](https://actix.rs)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [Redis-rs Documentation](https://github.com/redis-rs/redis-rs)
- [Rust Book](https://doc.rust-lang.org/book/)

---

Happy coding! 🚀
