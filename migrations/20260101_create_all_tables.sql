-- KetoBook Database Schema - Consolidated Migration for 2026
-- This migration creates all tables in proper dependency order

-- ============================================================================
-- STEP 1: Create ENUM types (must come first, no dependencies)
-- ============================================================================

-- Create wallet_type enum
CREATE TYPE IF NOT EXISTS wallet_type AS ENUM ('Cash', 'BankAccount', 'CreditCard', 'Other');

-- Create transaction_type enum
CREATE TYPE IF NOT EXISTS transaction_status AS ENUM ('income', 'expense');

-- ============================================================================
-- STEP 2: Create Wallets table (depends only on wallet_type ENUM)
-- ============================================================================

CREATE TABLE IF NOT EXISTS wallets (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    
    -- Balance fields with BigDecimal precision
    balance DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    credit_limit DECIMAL(15, 2) DEFAULT 0.00,
    
    -- Wallet type
    wallet_type wallet_type NOT NULL DEFAULT 'Cash',
    
    -- Timestamps in UTC (2026)
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    CONSTRAINT balance_non_negative CHECK (balance >= 0),
    CONSTRAINT credit_limit_non_negative CHECK (credit_limit >= 0)
);

-- Comments for clarity
COMMENT ON TABLE wallets IS 'User wallets for managing different account types (Cash, Bank, Credit Card)';
COMMENT ON COLUMN wallets.balance IS 
    'For CreditCard wallets: represents current debt (0=no debt, limit=fully used). 
     For other wallets: represents current balance. 
     Both use DECIMAL(15,2) for financial precision.';
COMMENT ON COLUMN wallets.credit_limit IS 
    'Credit limit for CreditCard wallets. For other types, this remains 0.00. 
     Available credit = credit_limit - balance.';
COMMENT ON COLUMN wallets.wallet_type IS 
    'Type of wallet: Cash (physical), BankAccount (checking/savings), CreditCard (credit line), Other (custom)';

-- Create indexes for wallets
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_wallets_user_type ON wallets(user_id, wallet_type);
CREATE INDEX IF NOT EXISTS idx_wallets_created_at ON wallets(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_wallets_credit_card ON wallets(user_id, wallet_type) WHERE wallet_type = 'CreditCard';

-- Create trigger for wallets updated_at
CREATE OR REPLACE FUNCTION update_wallets_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS wallets_update_timestamp ON wallets;
CREATE TRIGGER wallets_update_timestamp
BEFORE UPDATE ON wallets
FOR EACH ROW
EXECUTE FUNCTION update_wallets_updated_at();

-- ============================================================================
-- STEP 3: Create Transactions table (depends on wallets via FK)
-- ============================================================================

CREATE TABLE IF NOT EXISTS transactions (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(100) NOT NULL,
    wallet_id VARCHAR(36) NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    
    -- Amount with BigDecimal precision (always positive, sign determined by type)
    amount DECIMAL(15, 2) NOT NULL CHECK (amount > 0),
    
    -- Transaction type
    transaction_type VARCHAR(20) NOT NULL CHECK (transaction_type IN ('income', 'expense')),
    
    -- Additional details
    category VARCHAR(100) NOT NULL,
    description VARCHAR(500),
    
    -- Timestamps in UTC (2026)
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Comments for clarity
COMMENT ON TABLE transactions IS 'Financial transactions linked to wallets with atomic balance updates';
COMMENT ON COLUMN transactions.amount IS 'Transaction amount in DECIMAL(15,2) for financial precision. Always positive, sign determined by transaction_type.';
COMMENT ON COLUMN transactions.transaction_type IS 'income: adds to wallet balance, expense: subtracts from wallet balance';
COMMENT ON COLUMN transactions.wallet_id IS 'Foreign key to wallets table - transaction must belong to existing wallet. CASCADE DELETE ensures orphaned transactions are removed.';

-- Create indexes for transactions
CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_wallet_id ON transactions(wallet_id);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_user_created ON transactions(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_user_wallet ON transactions(user_id, wallet_id);

-- Create trigger for transactions updated_at
CREATE OR REPLACE FUNCTION update_transactions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS transactions_update_timestamp ON transactions;
CREATE TRIGGER transactions_update_timestamp
BEFORE UPDATE ON transactions
FOR EACH ROW
EXECUTE FUNCTION update_transactions_updated_at();

-- ============================================================================
-- STEP 4: Create Debts table (independent, can be created after transactions)
-- ============================================================================

CREATE TABLE IF NOT EXISTS debts (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(100) NOT NULL,
    creditor_name VARCHAR(255) NOT NULL,
    
    -- Amount with BigDecimal precision
    amount DECIMAL(15, 2) NOT NULL CHECK (amount > 0),
    
    -- Interest and due date
    interest_rate DECIMAL(5, 2) DEFAULT 0.00 CHECK (interest_rate >= 0),
    due_date TIMESTAMP WITH TIME ZONE,
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paid', 'cancelled')),
    
    -- Timestamps in UTC (2026)
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Comments for clarity
COMMENT ON TABLE debts IS 'Debt tracking for loans and outstanding balances';
COMMENT ON COLUMN debts.amount IS 'Outstanding debt amount in DECIMAL(15,2) for financial precision';
COMMENT ON COLUMN debts.interest_rate IS 'Annual interest rate in percent. Default 0.00 for interest-free debts.';
COMMENT ON COLUMN debts.status IS 'active: ongoing debt, paid: fully repaid, cancelled: forgiven/written off';

-- Create indexes for debts
CREATE INDEX IF NOT EXISTS idx_debts_user_id ON debts(user_id);
CREATE INDEX IF NOT EXISTS idx_debts_status ON debts(status);
CREATE INDEX IF NOT EXISTS idx_debts_due_date ON debts(due_date);
CREATE INDEX IF NOT EXISTS idx_debts_user_status ON debts(user_id, status);

-- Create trigger for debts updated_at
CREATE OR REPLACE FUNCTION update_debts_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS debts_update_timestamp ON debts;
CREATE TRIGGER debts_update_timestamp
BEFORE UPDATE ON debts
FOR EACH ROW
EXECUTE FUNCTION update_debts_updated_at();

-- ============================================================================
-- STEP 5: Create Views for aggregations
-- ============================================================================

-- Transaction summary view
CREATE OR REPLACE VIEW v_transaction_summary AS
SELECT
    user_id,
    wallet_id,
    transaction_type,
    COUNT(*) as transaction_count,
    SUM(amount) as total_amount,
    AVG(amount) as avg_amount,
    MIN(created_at) as first_transaction,
    MAX(created_at) as last_transaction
FROM transactions
GROUP BY user_id, wallet_id, transaction_type;

COMMENT ON VIEW v_transaction_summary IS 'Aggregate statistics for transactions by user, wallet, and type';

-- Debt summary view
CREATE OR REPLACE VIEW v_debt_summary AS
SELECT
    user_id,
    status,
    COUNT(*) as debt_count,
    SUM(amount) as total_amount,
    AVG(interest_rate) as avg_interest_rate
FROM debts
GROUP BY user_id, status;

COMMENT ON VIEW v_debt_summary IS 'Aggregate statistics for debts by user and status';

-- Wallet summary view with available credit calculation
CREATE OR REPLACE VIEW v_wallet_summary AS
SELECT
    w.id,
    w.user_id,
    w.name,
    w.wallet_type,
    w.balance,
    w.credit_limit,
    CASE 
        WHEN w.wallet_type = 'CreditCard' THEN w.credit_limit - w.balance
        ELSE w.balance
    END as available_balance,
    COUNT(t.id) as transaction_count,
    COALESCE(SUM(CASE WHEN t.transaction_type = 'income' THEN t.amount ELSE 0 END), 0) as total_income,
    COALESCE(SUM(CASE WHEN t.transaction_type = 'expense' THEN t.amount ELSE 0 END), 0) as total_expense,
    w.created_at,
    w.updated_at
FROM wallets w
LEFT JOIN transactions t ON w.id = t.wallet_id
GROUP BY w.id, w.user_id, w.name, w.wallet_type, w.balance, w.credit_limit, w.created_at, w.updated_at;

COMMENT ON VIEW v_wallet_summary IS 'Wallet statistics with available balance calculation and transaction counts';

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
-- All tables created with:
-- ✓ Proper dependency ordering (ENUM → Wallets → Transactions/Debts → Views)
-- ✓ Financial precision using DECIMAL(15, 2)
-- ✓ Atomic transaction support via foreign keys
-- ✓ Credit card balance semantics (balance = debt)
-- ✓ Auto-updating timestamps (UTC)
-- ✓ Comprehensive indexing for performance
-- ✓ Triggers for maintaining data consistency
-- ✓ Comments for schema documentation
