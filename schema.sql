-- KetoBook Finance Management API Database Schema
-- PostgreSQL 12+

-- Create extensions if needed
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ==================== Transactions Table ====================
-- Stores personal income and expense transactions
CREATE TABLE IF NOT EXISTS transactions (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    amount DECIMAL(15, 2) NOT NULL CHECK (amount > 0),
    transaction_type VARCHAR(50) NOT NULL CHECK (transaction_type IN ('income', 'expense')),
    category VARCHAR(100) NOT NULL,
    description VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for optimal query performance
CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_created_at ON transactions(created_at DESC);
CREATE INDEX idx_transactions_user_created ON transactions(user_id, created_at DESC);

-- ==================== Debts Table ====================
-- Stores information about loans and debts
CREATE TABLE IF NOT EXISTS debts (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    creditor_name VARCHAR(255) NOT NULL,
    amount DECIMAL(15, 2) NOT NULL CHECK (amount > 0),
    interest_rate DECIMAL(5, 2) NOT NULL DEFAULT 0.0 CHECK (interest_rate >= 0),
    due_date TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paid')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for optimal query performance
CREATE INDEX idx_debts_user_id ON debts(user_id);
CREATE INDEX idx_debts_status ON debts(status);
CREATE INDEX idx_debts_due_date ON debts(due_date);
CREATE INDEX idx_debts_user_status ON debts(user_id, status);

-- ==================== Triggers ====================
-- Automatically update updated_at timestamp on transactions
CREATE OR REPLACE FUNCTION update_transactions_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER transactions_update_timestamp
BEFORE UPDATE ON transactions
FOR EACH ROW
EXECUTE FUNCTION update_transactions_timestamp();

-- Automatically update updated_at timestamp on debts
CREATE OR REPLACE FUNCTION update_debts_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER debts_update_timestamp
BEFORE UPDATE ON debts
FOR EACH ROW
EXECUTE FUNCTION update_debts_timestamp();

-- ==================== Views (Optional) ====================
-- View for transaction summary by user
CREATE OR REPLACE VIEW v_transaction_summary AS
SELECT 
    user_id,
    transaction_type,
    category,
    COUNT(*) as transaction_count,
    SUM(amount) as total_amount,
    AVG(amount) as average_amount,
    MIN(amount) as min_amount,
    MAX(amount) as max_amount,
    DATE_TRUNC('month', created_at) as month
FROM transactions
GROUP BY user_id, transaction_type, category, DATE_TRUNC('month', created_at);

-- View for debt summary by user
CREATE OR REPLACE VIEW v_debt_summary AS
SELECT 
    user_id,
    status,
    COUNT(*) as debt_count,
    SUM(amount) as total_amount,
    AVG(interest_rate) as average_interest_rate,
    MIN(due_date) as earliest_due_date,
    MAX(due_date) as latest_due_date
FROM debts
GROUP BY user_id, status;
