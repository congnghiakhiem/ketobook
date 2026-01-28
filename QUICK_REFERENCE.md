# KetoBook - Quick Reference Card

## 🚀 Start Here

```bash
# 1. Copy environment template
cp .env.example .env

# 2. Edit .env with your credentials
nano .env

# 3. Create database and run schema
createdb ketobook_db
psql ketobook_db < schema.sql

# 4. Run the server
cargo run

# 5. Test in another terminal
curl http://localhost:8080/health
```

---

## 📍 File Navigation

| Need | File |
|------|------|
| **Getting Started** | SCAFFOLDING_SUMMARY.md |
| **Installation** | SETUP.md |
| **Full Docs** | README.md |
| **API Details** | API_REFERENCE.md |
| **File Structure** | PROJECT_STRUCTURE.md |
| **Status Check** | IMPLEMENTATION_CHECKLIST.md |
| **All Resources** | INDEX.md |

---

## 🔌 Database URLs

```env
DATABASE_URL=postgresql://user:password@localhost:5432/ketobook_db
REDIS_URL=redis://127.0.0.1:6379
```

---

## 🧪 Test API

### Health Check
```bash
curl http://localhost:8080/health
```

### Create Transaction
```bash
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_123",
    "amount": 50.00,
    "transaction_type": "expense",
    "category": "groceries",
    "description": "Weekly shopping"
  }'
```

### Get All Transactions
```bash
curl http://localhost:8080/api/transactions/user/user_123
```

### Full Test Suite
```bash
bash test_api.sh              # Linux/Mac
powershell test_api.ps1       # Windows
```

---

## 📦 Key Dependencies

| Package | Purpose |
|---------|---------|
| actix-web | Web framework |
| tokio | Async runtime |
| sqlx | Database driver |
| redis | Cache driver |
| serde | JSON serialization |
| uuid | ID generation |
| chrono | Date/time |
| dotenv | Configuration |

---

## 📚 Module Map

```
main.rs
├── Loads config.rs
├── Creates db from db.rs
├── Creates cache from cache.rs
├── Configures transactions.rs routes
└── Configures debts.rs routes

models.rs
└── Shared data structures

cache.rs
└── Redis + cache-aside pattern

transactions.rs
└── CRUD handlers

debts.rs
└── CRUD handlers
```

---

## 🗂️ Project Structure

```
src/
├── main.rs         ← Server entry point
├── config.rs       ← Load .env
├── models.rs       ← Data types
├── db.rs           ← PostgreSQL
├── cache.rs        ← Redis
├── transactions.rs ← Endpoints
└── debts.rs        ← Endpoints
```

---

## 🔄 Cache Flow

```
Request
  ↓
Redis? → Yes → Return data
  ↓ No
PostgreSQL
  ↓
Cache (1 hour)
  ↓
Return data
```

---

## 🌐 API Endpoints

### Transactions
```
GET    /api/transactions/user/{id}
GET    /api/transactions/{uid}/{tid}
POST   /api/transactions
PUT    /api/transactions/{uid}/{tid}
DELETE /api/transactions/{uid}/{tid}
```

### Debts
```
GET    /api/debts/user/{id}
GET    /api/debts/{uid}/{did}
POST   /api/debts
PUT    /api/debts/{uid}/{did}
DELETE /api/debts/{uid}/{did}
```

---

## 🛠️ Common Commands

```bash
cargo run              # Run server
cargo build            # Build binary
cargo build --release  # Optimized build
cargo check            # Check syntax
cargo fmt              # Format code
cargo clippy           # Lint code
cargo test             # Run tests
cargo clean            # Remove artifacts
```

---

## 📊 Database Tables

### transactions
- id (UUID)
- user_id
- amount (decimal)
- transaction_type (income/expense)
- category
- description
- created_at, updated_at

### debts
- id (UUID)
- user_id
- creditor_name
- amount (decimal)
- interest_rate
- due_date
- status (active/paid)
- created_at, updated_at

---

## 🔑 Environment Variables

```env
DATABASE_URL        # PostgreSQL connection
REDIS_URL           # Redis connection
SERVER_HOST         # Default: 127.0.0.1
SERVER_PORT         # Default: 8080
RUST_LOG            # Default: info
```

---

## 💻 Typical Development Cycle

```
1. Edit src files
2. cargo check          ← Quick syntax check
3. cargo run            ← Test locally
4. curl/test_api.sh     ← Verify API
5. cargo clippy         ← Check for issues
6. git commit           ← Save progress
```

---

## 🚨 Common Issues

| Error | Solution |
|-------|----------|
| Connection refused (DB) | Check PostgreSQL is running, verify DATABASE_URL |
| Redis error | Check Redis is running, verify REDIS_URL |
| Port 8080 in use | Change SERVER_PORT in .env |
| Database not found | Run `createdb ketobook_db` |
| Tables not found | Run `psql ketobook_db < schema.sql` |

---

## 📈 Performance Tips

- Redis caching active (1-hour TTL)
- Connection pooling enabled (5 max)
- Database indexes on user_id
- Async throughout (no blocking)

---

## 🔐 Security Reminders

- ✅ Credentials in .env (git-ignored)
- ⚠️ Add authentication before production
- ⚠️ Add input validation
- ⚠️ Use HTTPS in production
- ⚠️ Enable rate limiting

---

## 📝 Response Format

### Success
```json
{
  "success": true,
  "data": { /* entity */ },
  "error": null
}
```

### Error
```json
{
  "success": false,
  "data": null,
  "error": "Description"
}
```

---

## 🎯 Next Steps

1. ✅ Read SCAFFOLDING_SUMMARY.md
2. ✅ Follow SETUP.md
3. ✅ Run `cargo run`
4. ✅ Test with test_api.sh
5. ⏳ Add authentication
6. ⏳ Add validation
7. ⏳ Deploy to production

---

## 📞 Quick Links

- **Documentation**: INDEX.md
- **API Reference**: API_REFERENCE.md  
- **Installation**: SETUP.md
- **Project Overview**: README.md

---

**Status**: ✅ Ready to code!
**Start**: SCAFFOLDING_SUMMARY.md
**Then**: SETUP.md
