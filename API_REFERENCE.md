# KetoBook API Reference

Complete API documentation for the KetoBook Finance Management API.

## Base URL
```
http://localhost:8080
```

## Response Format

All API responses follow a standard JSON format:

### Success Response (2xx)
```json
{
  "success": true,
  "data": { /* response data */ },
  "error": null
}
```

### Error Response (4xx, 5xx)
```json
{
  "success": false,
  "data": null,
  "error": "Description of what went wrong"
}
```

---

## Health Check

### GET /health

Check if the API server is running and healthy.

**Response:** `200 OK`
```json
{
  "status": "healthy",
  "timestamp": "2025-01-28T10:30:00Z"
}
```

---

## Transactions API

### Data Model

```typescript
interface Transaction {
  id: string;                    // UUID v4, auto-generated
  user_id: string;              // User identifier
  amount: number;               // > 0, decimal with 2 places
  transaction_type: string;     // "income" | "expense"
  category: string;             // e.g., "groceries", "salary"
  description: string;          // Optional details
  created_at: string;           // ISO 8601 timestamp
  updated_at: string;           // ISO 8601 timestamp
}
```

---

### GET /api/transactions/user/{user_id}

Retrieve all transactions for a specific user.

**Parameters:**
- `user_id` (path) - User identifier

**Query Parameters:**
- None (pagination coming soon)

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "user_id": "user_123",
      "amount": 45.50,
      "transaction_type": "expense",
      "category": "groceries",
      "description": "Weekly groceries",
      "created_at": "2025-01-28T10:00:00Z",
      "updated_at": "2025-01-28T10:00:00Z"
    }
  ],
  "error": null
}
```

**Error Responses:**
- `500 Internal Server Error` - Database or cache error

---

### GET /api/transactions/{user_id}/{transaction_id}

Retrieve a specific transaction by ID.

**Parameters:**
- `user_id` (path) - User identifier
- `transaction_id` (path) - Transaction UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "user_id": "user_123",
    "amount": 45.50,
    "transaction_type": "expense",
    "category": "groceries",
    "description": "Weekly groceries",
    "created_at": "2025-01-28T10:00:00Z",
    "updated_at": "2025-01-28T10:00:00Z"
  },
  "error": null
}
```

**Error Responses:**
- `404 Not Found` - Transaction not found for this user
- `500 Internal Server Error` - Database or cache error

---

### POST /api/transactions

Create a new transaction.

**Request Body:**
```json
{
  "user_id": "user_123",
  "amount": 45.50,
  "transaction_type": "expense",
  "category": "groceries",
  "description": "Weekly grocery shopping"
}
```

**Validation:**
- `user_id`: Required, string
- `amount`: Required, number > 0
- `transaction_type`: Required, must be "income" or "expense"
- `category`: Required, string (max 100 chars)
- `description`: Optional, string (max 500 chars)

**Response:** `201 Created`
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "user_id": "user_123",
    "amount": 45.50,
    "transaction_type": "expense",
    "category": "groceries",
    "description": "Weekly grocery shopping",
    "created_at": "2025-01-28T10:05:00Z",
    "updated_at": "2025-01-28T10:05:00Z"
  },
  "error": null
}
```

**Error Responses:**
- `400 Bad Request` - Invalid request data
- `500 Internal Server Error` - Database error

---

### PUT /api/transactions/{user_id}/{transaction_id}

Update an existing transaction.

**Parameters:**
- `user_id` (path) - User identifier
- `transaction_id` (path) - Transaction UUID

**Request Body:** (all fields optional)
```json
{
  "amount": 55.75,
  "category": "food",
  "description": "Updated description"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "user_id": "user_123",
    "amount": 55.75,
    "transaction_type": "expense",
    "category": "food",
    "description": "Updated description",
    "created_at": "2025-01-28T10:00:00Z",
    "updated_at": "2025-01-28T10:10:00Z"
  },
  "error": null
}
```

**Error Responses:**
- `404 Not Found` - Transaction not found for this user
- `500 Internal Server Error` - Database error

---

### DELETE /api/transactions/{user_id}/{transaction_id}

Delete a transaction.

**Parameters:**
- `user_id` (path) - User identifier
- `transaction_id` (path) - Transaction UUID

**Response:** `204 No Content` (empty body)

**Error Responses:**
- `404 Not Found` - Transaction not found for this user
- `500 Internal Server Error` - Database error

---

## Debts API

### Data Model

```typescript
interface Debt {
  id: string;                    // UUID v4, auto-generated
  user_id: string;              // User identifier
  creditor_name: string;        // Lender's name
  amount: number;               // > 0, decimal with 2 places
  interest_rate: number;        // >= 0, decimal with 2 places
  due_date: string;             // ISO 8601 timestamp
  status: string;               // "active" | "paid"
  created_at: string;           // ISO 8601 timestamp
  updated_at: string;           // ISO 8601 timestamp
}
```

---

### GET /api/debts/user/{user_id}

Retrieve all debts for a specific user.

**Parameters:**
- `user_id` (path) - User identifier

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440100",
      "user_id": "user_123",
      "creditor_name": "Bank of America",
      "amount": 5000.00,
      "interest_rate": 18.5,
      "due_date": "2025-12-31T23:59:59Z",
      "status": "active",
      "created_at": "2025-01-28T10:00:00Z",
      "updated_at": "2025-01-28T10:00:00Z"
    }
  ],
  "error": null
}
```

**Error Responses:**
- `500 Internal Server Error` - Database or cache error

---

### GET /api/debts/{user_id}/{debt_id}

Retrieve a specific debt by ID.

**Parameters:**
- `user_id` (path) - User identifier
- `debt_id` (path) - Debt UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440100",
    "user_id": "user_123",
    "creditor_name": "Bank of America",
    "amount": 5000.00,
    "interest_rate": 18.5,
    "due_date": "2025-12-31T23:59:59Z",
    "status": "active",
    "created_at": "2025-01-28T10:00:00Z",
    "updated_at": "2025-01-28T10:00:00Z"
  },
  "error": null
}
```

**Error Responses:**
- `404 Not Found` - Debt not found for this user
- `500 Internal Server Error` - Database or cache error

---

### POST /api/debts

Create a new debt.

**Request Body:**
```json
{
  "user_id": "user_123",
  "creditor_name": "Bank of America",
  "amount": 5000.00,
  "interest_rate": 18.5,
  "due_date": "2025-12-31T23:59:59Z"
}
```

**Validation:**
- `user_id`: Required, string
- `creditor_name`: Required, string (max 255 chars)
- `amount`: Required, number > 0
- `interest_rate`: Required, number >= 0
- `due_date`: Required, ISO 8601 timestamp (future date)

**Response:** `201 Created`
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440101",
    "user_id": "user_123",
    "creditor_name": "Bank of America",
    "amount": 5000.00,
    "interest_rate": 18.5,
    "due_date": "2025-12-31T23:59:59Z",
    "status": "active",
    "created_at": "2025-01-28T10:05:00Z",
    "updated_at": "2025-01-28T10:05:00Z"
  },
  "error": null
}
```

**Error Responses:**
- `400 Bad Request` - Invalid request data
- `500 Internal Server Error` - Database error

---

### PUT /api/debts/{user_id}/{debt_id}

Update an existing debt.

**Parameters:**
- `user_id` (path) - User identifier
- `debt_id` (path) - Debt UUID

**Request Body:** (all fields optional)
```json
{
  "creditor_name": "Bank of America",
  "amount": 4500.00,
  "interest_rate": 18.5,
  "due_date": "2025-12-31T23:59:59Z",
  "status": "active"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440100",
    "user_id": "user_123",
    "creditor_name": "Bank of America",
    "amount": 4500.00,
    "interest_rate": 18.5,
    "due_date": "2025-12-31T23:59:59Z",
    "status": "active",
    "created_at": "2025-01-28T10:00:00Z",
    "updated_at": "2025-01-28T10:15:00Z"
  },
  "error": null
}
```

**Error Responses:**
- `404 Not Found` - Debt not found for this user
- `500 Internal Server Error` - Database error

---

### DELETE /api/debts/{user_id}/{debt_id}

Delete a debt.

**Parameters:**
- `user_id` (path) - User identifier
- `debt_id` (path) - Debt UUID

**Response:** `204 No Content` (empty body)

**Error Responses:**
- `404 Not Found` - Debt not found for this user
- `500 Internal Server Error` - Database error

---

## Example Usage

### Create and Manage a Transaction
```bash
# Create expense
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_123",
    "amount": 50.00,
    "transaction_type": "expense",
    "category": "groceries",
    "description": "Weekly shopping"
  }'

# Retrieve all transactions
curl http://localhost:8080/api/transactions/user/user_123

# Update transaction
curl -X PUT http://localhost:8080/api/transactions/user_123/txn_id \
  -H "Content-Type: application/json" \
  -d '{"amount": 55.00}'

# Delete transaction
curl -X DELETE http://localhost:8080/api/transactions/user_123/txn_id
```

### Manage Debts
```bash
# Create debt
curl -X POST http://localhost:8080/api/debts \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_123",
    "creditor_name": "Credit Card Co",
    "amount": 3000.00,
    "interest_rate": 19.99,
    "due_date": "2025-12-31T23:59:59Z"
  }'

# Mark debt as paid
curl -X PUT http://localhost:8080/api/debts/user_123/debt_id \
  -H "Content-Type: application/json" \
  -d '{"status": "paid"}'
```

---

## Rate Limits

Currently not implemented. Add rate limiting middleware for production.

---

## Authentication

Currently not implemented. Add JWT token validation for production.

---

## Error Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Successful GET request |
| 201 | Created | Successful POST request |
| 204 | No Content | Successful DELETE request |
| 400 | Bad Request | Invalid request data or validation failed |
| 404 | Not Found | Resource not found |
| 500 | Internal Server Error | Server error, check logs |

---

## Performance Notes

- All list endpoints use Redis caching with 1-hour TTL
- Cache is invalidated on create/update/delete operations
- Database queries use connection pooling (max 5 concurrent)
- Timestamps are in UTC (ISO 8601 format)

---

Last Updated: January 28, 2025
