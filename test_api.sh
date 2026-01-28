#!/usr/bin/env bash

# KetoBook API Test Script
# This script demonstrates all API endpoints with example requests

API_BASE="http://localhost:8080"
USER_ID="user_123"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== KetoBook API Test Script ===${NC}\n"

# 1. Health Check
echo -e "${YELLOW}1. Testing Health Check${NC}"
curl -s "$API_BASE/health" | jq .
echo -e "\n"

# 2. Create Transaction 1
echo -e "${YELLOW}2. Creating First Transaction (Expense)${NC}"
TRANS1=$(curl -s -X POST "$API_BASE/api/transactions" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "'$USER_ID'",
    "amount": 45.50,
    "transaction_type": "expense",
    "category": "groceries",
    "description": "Weekly groceries at Whole Foods"
  }')
echo $TRANS1 | jq .
TRANS1_ID=$(echo $TRANS1 | jq -r '.data.id')
echo "Transaction ID: $TRANS1_ID"
echo -e "\n"

# 3. Create Transaction 2
echo -e "${YELLOW}3. Creating Second Transaction (Income)${NC}"
TRANS2=$(curl -s -X POST "$API_BASE/api/transactions" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "'$USER_ID'",
    "amount": 3000.00,
    "transaction_type": "income",
    "category": "salary",
    "description": "Monthly salary deposit"
  }')
echo $TRANS2 | jq .
TRANS2_ID=$(echo $TRANS2 | jq -r '.data.id')
echo "Transaction ID: $TRANS2_ID"
echo -e "\n"

# 4. Get All Transactions
echo -e "${YELLOW}4. Fetching All Transactions for User${NC}"
curl -s "$API_BASE/api/transactions/user/$USER_ID" | jq .
echo -e "\n"

# 5. Get Single Transaction
echo -e "${YELLOW}5. Fetching Single Transaction${NC}"
curl -s "$API_BASE/api/transactions/$USER_ID/$TRANS1_ID" | jq .
echo -e "\n"

# 6. Update Transaction
echo -e "${YELLOW}6. Updating Transaction Amount${NC}"
curl -s -X PUT "$API_BASE/api/transactions/$USER_ID/$TRANS1_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 55.75,
    "description": "Weekly groceries (updated)"
  }' | jq .
echo -e "\n"

# 7. Create Debt 1
echo -e "${YELLOW}7. Creating First Debt${NC}"
DEBT1=$(curl -s -X POST "$API_BASE/api/debts" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "'$USER_ID'",
    "creditor_name": "Bank of America",
    "amount": 5000.00,
    "interest_rate": 18.5,
    "due_date": "2025-12-31T23:59:59Z"
  }')
echo $DEBT1 | jq .
DEBT1_ID=$(echo $DEBT1 | jq -r '.data.id')
echo "Debt ID: $DEBT1_ID"
echo -e "\n"

# 8. Create Debt 2
echo -e "${YELLOW}8. Creating Second Debt${NC}"
DEBT2=$(curl -s -X POST "$API_BASE/api/debts" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "'$USER_ID'",
    "creditor_name": "Credit Card Co",
    "amount": 2500.00,
    "interest_rate": 22.0,
    "due_date": "2025-06-30T23:59:59Z"
  }')
echo $DEBT2 | jq .
DEBT2_ID=$(echo $DEBT2 | jq -r '.data.id')
echo "Debt ID: $DEBT2_ID"
echo -e "\n"

# 9. Get All Debts
echo -e "${YELLOW}9. Fetching All Debts for User${NC}"
curl -s "$API_BASE/api/debts/user/$USER_ID" | jq .
echo -e "\n"

# 10. Get Single Debt
echo -e "${YELLOW}10. Fetching Single Debt${NC}"
curl -s "$API_BASE/api/debts/$USER_ID/$DEBT1_ID" | jq .
echo -e "\n"

# 11. Update Debt
echo -e "${YELLOW}11. Updating Debt (Pay Down)${NC}"
curl -s -X PUT "$API_BASE/api/debts/$USER_ID/$DEBT1_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 4500.00,
    "status": "active"
  }' | jq .
echo -e "\n"

# 12. Delete Transaction
echo -e "${YELLOW}12. Deleting a Transaction${NC}"
curl -s -X DELETE "$API_BASE/api/transactions/$USER_ID/$TRANS2_ID" -w "\nStatus: %{http_code}\n"
echo -e "\n"

# 13. Delete Debt
echo -e "${YELLOW}13. Deleting a Debt${NC}"
curl -s -X DELETE "$API_BASE/api/debts/$USER_ID/$DEBT2_ID" -w "\nStatus: %{http_code}\n"
echo -e "\n"

# 14. Final State Check
echo -e "${YELLOW}14. Final State - Remaining Transactions${NC}"
curl -s "$API_BASE/api/transactions/user/$USER_ID" | jq .
echo -e "\n"

echo -e "${YELLOW}15. Final State - Remaining Debts${NC}"
curl -s "$API_BASE/api/debts/user/$USER_ID" | jq .
echo -e "\n"

echo -e "${GREEN}=== Test Complete ===${NC}"
