-- Create aggregation view for transactions
CREATE OR REPLACE VIEW v_transaction_summary AS
SELECT 
    user_id,
    COUNT(*) as total_transactions,
    SUM(CASE WHEN transaction_type = 'income' THEN amount ELSE 0 END) as total_income,
    SUM(CASE WHEN transaction_type = 'expense' THEN amount ELSE 0 END) as total_expense,
    SUM(CASE WHEN transaction_type = 'income' THEN amount ELSE -amount END) as net_balance,
    MIN(created_at) as first_transaction,
    MAX(created_at) as last_transaction
FROM transactions
GROUP BY user_id;

-- Create aggregation view for debts
CREATE OR REPLACE VIEW v_debt_summary AS
SELECT 
    user_id,
    COUNT(*) as total_debts,
    COUNT(CASE WHEN status = 'active' THEN 1 END) as active_debts,
    COUNT(CASE WHEN status = 'paid' THEN 1 END) as paid_debts,
    SUM(amount) as total_debt_amount,
    SUM(CASE WHEN status = 'active' THEN amount ELSE 0 END) as active_debt_amount,
    AVG(interest_rate) as avg_interest_rate,
    MAX(due_date) as latest_due_date
FROM debts
GROUP BY user_id;
