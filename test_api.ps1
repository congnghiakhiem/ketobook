# KetoBook API Test Script (PowerShell)
# This script demonstrates all API endpoints with example requests

$apiBase = "http://localhost:8080"
$userId = "user_123"

Write-Host "=== KetoBook API Test Script ===" -ForegroundColor Cyan
Write-Host ""

# Helper function to pretty-print JSON
function Format-Json {
    param([Parameter(ValueFromPipeline)]$Json)
    $Json | ConvertFrom-Json | ConvertTo-Json -Depth 4
}

# 1. Health Check
Write-Host "1. Testing Health Check" -ForegroundColor Yellow
$response = Invoke-WebRequest "$apiBase/health" -UseBasicParsing
$response.Content | Format-Json
Write-Host ""

# 2. Create Transaction 1
Write-Host "2. Creating First Transaction (Expense)" -ForegroundColor Yellow
$trans1Body = @{
    user_id = $userId
    amount = 45.50
    transaction_type = "expense"
    category = "groceries"
    description = "Weekly groceries at Whole Foods"
} | ConvertTo-Json

$trans1Response = Invoke-WebRequest "$apiBase/api/transactions" `
    -Method Post `
    -Headers @{'Content-Type' = 'application/json'} `
    -Body $trans1Body `
    -UseBasicParsing

$trans1 = $trans1Response.Content | ConvertFrom-Json
$trans1 | ConvertTo-Json | Write-Host
$trans1Id = $trans1.data.id
Write-Host "Transaction ID: $trans1Id" -ForegroundColor Green
Write-Host ""

# 3. Create Transaction 2
Write-Host "3. Creating Second Transaction (Income)" -ForegroundColor Yellow
$trans2Body = @{
    user_id = $userId
    amount = 3000.00
    transaction_type = "income"
    category = "salary"
    description = "Monthly salary deposit"
} | ConvertTo-Json

$trans2Response = Invoke-WebRequest "$apiBase/api/transactions" `
    -Method Post `
    -Headers @{'Content-Type' = 'application/json'} `
    -Body $trans2Body `
    -UseBasicParsing

$trans2 = $trans2Response.Content | ConvertFrom-Json
$trans2 | ConvertTo-Json | Write-Host
$trans2Id = $trans2.data.id
Write-Host "Transaction ID: $trans2Id" -ForegroundColor Green
Write-Host ""

# 4. Get All Transactions
Write-Host "4. Fetching All Transactions for User" -ForegroundColor Yellow
$transResponse = Invoke-WebRequest "$apiBase/api/transactions/user/$userId" -UseBasicParsing
$transResponse.Content | Format-Json
Write-Host ""

# 5. Get Single Transaction
Write-Host "5. Fetching Single Transaction" -ForegroundColor Yellow
$singleTransResponse = Invoke-WebRequest "$apiBase/api/transactions/$userId/$trans1Id" -UseBasicParsing
$singleTransResponse.Content | Format-Json
Write-Host ""

# 6. Update Transaction
Write-Host "6. Updating Transaction Amount" -ForegroundColor Yellow
$updateTransBody = @{
    amount = 55.75
    description = "Weekly groceries (updated)"
} | ConvertTo-Json

$updateTransResponse = Invoke-WebRequest "$apiBase/api/transactions/$userId/$trans1Id" `
    -Method Put `
    -Headers @{'Content-Type' = 'application/json'} `
    -Body $updateTransBody `
    -UseBasicParsing

$updateTransResponse.Content | Format-Json
Write-Host ""

# 7. Create Debt 1
Write-Host "7. Creating First Debt" -ForegroundColor Yellow
$debt1Body = @{
    user_id = $userId
    creditor_name = "Bank of America"
    amount = 5000.00
    interest_rate = 18.5
    due_date = "2025-12-31T23:59:59Z"
} | ConvertTo-Json

$debt1Response = Invoke-WebRequest "$apiBase/api/debts" `
    -Method Post `
    -Headers @{'Content-Type' = 'application/json'} `
    -Body $debt1Body `
    -UseBasicParsing

$debt1 = $debt1Response.Content | ConvertFrom-Json
$debt1 | ConvertTo-Json | Write-Host
$debt1Id = $debt1.data.id
Write-Host "Debt ID: $debt1Id" -ForegroundColor Green
Write-Host ""

# 8. Create Debt 2
Write-Host "8. Creating Second Debt" -ForegroundColor Yellow
$debt2Body = @{
    user_id = $userId
    creditor_name = "Credit Card Co"
    amount = 2500.00
    interest_rate = 22.0
    due_date = "2025-06-30T23:59:59Z"
} | ConvertTo-Json

$debt2Response = Invoke-WebRequest "$apiBase/api/debts" `
    -Method Post `
    -Headers @{'Content-Type' = 'application/json'} `
    -Body $debt2Body `
    -UseBasicParsing

$debt2 = $debt2Response.Content | ConvertFrom-Json
$debt2 | ConvertTo-Json | Write-Host
$debt2Id = $debt2.data.id
Write-Host "Debt ID: $debt2Id" -ForegroundColor Green
Write-Host ""

# 9. Get All Debts
Write-Host "9. Fetching All Debts for User" -ForegroundColor Yellow
$debtsResponse = Invoke-WebRequest "$apiBase/api/debts/user/$userId" -UseBasicParsing
$debtsResponse.Content | Format-Json
Write-Host ""

# 10. Get Single Debt
Write-Host "10. Fetching Single Debt" -ForegroundColor Yellow
$singleDebtResponse = Invoke-WebRequest "$apiBase/api/debts/$userId/$debt1Id" -UseBasicParsing
$singleDebtResponse.Content | Format-Json
Write-Host ""

# 11. Update Debt
Write-Host "11. Updating Debt (Pay Down)" -ForegroundColor Yellow
$updateDebtBody = @{
    amount = 4500.00
    status = "active"
} | ConvertTo-Json

$updateDebtResponse = Invoke-WebRequest "$apiBase/api/debts/$userId/$debt1Id" `
    -Method Put `
    -Headers @{'Content-Type' = 'application/json'} `
    -Body $updateDebtBody `
    -UseBasicParsing

$updateDebtResponse.Content | Format-Json
Write-Host ""

# 12. Delete Transaction
Write-Host "12. Deleting a Transaction" -ForegroundColor Yellow
try {
    $deleteTransResponse = Invoke-WebRequest "$apiBase/api/transactions/$userId/$trans2Id" `
        -Method Delete `
        -UseBasicParsing
    Write-Host "Delete successful - Status: $($deleteTransResponse.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "Delete response: $($_.Exception.Response.StatusCode)" -ForegroundColor Green
}
Write-Host ""

# 13. Delete Debt
Write-Host "13. Deleting a Debt" -ForegroundColor Yellow
try {
    $deleteDebtResponse = Invoke-WebRequest "$apiBase/api/debts/$userId/$debt2Id" `
        -Method Delete `
        -UseBasicParsing
    Write-Host "Delete successful - Status: $($deleteDebtResponse.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "Delete response: $($_.Exception.Response.StatusCode)" -ForegroundColor Green
}
Write-Host ""

# 14. Final State Check - Transactions
Write-Host "14. Final State - Remaining Transactions" -ForegroundColor Yellow
$finalTransResponse = Invoke-WebRequest "$apiBase/api/transactions/user/$userId" -UseBasicParsing
$finalTransResponse.Content | Format-Json
Write-Host ""

# 15. Final State Check - Debts
Write-Host "15. Final State - Remaining Debts" -ForegroundColor Yellow
$finalDebtResponse = Invoke-WebRequest "$apiBase/api/debts/user/$userId" -UseBasicParsing
$finalDebtResponse.Content | Format-Json
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
