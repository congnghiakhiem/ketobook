-- KetoBook Initial Schema Migration (2026-01-28)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- STEP 1: Create ENUM types
DO $$ BEGIN
    CREATE TYPE wallet_type AS ENUM ('Cash', 'BankAccount', 'CreditCard', 'Other');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- STEP 2: Create wallets table (Sử dụng kiểu UUID)
CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    wallet_type wallet_type NOT NULL DEFAULT 'Cash',
    balance DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    credit_limit DECIMAL(15, 2) DEFAULT 0.00,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT balance_non_negative CHECK (balance >= 0),
    CONSTRAINT credit_limit_non_negative CHECK (credit_limit >= 0)
);

CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_wallets_wallet_type ON wallets(wallet_type);
CREATE INDEX IF NOT EXISTS idx_wallets_created_at ON wallets(created_at);

CREATE OR REPLACE FUNCTION update_wallets_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_wallets_updated_at ON wallets;
CREATE TRIGGER trigger_wallets_updated_at
    BEFORE UPDATE ON wallets
    FOR EACH ROW
    EXECUTE FUNCTION update_wallets_updated_at();

-- STEP 3: Create transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(100) NOT NULL,
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    amount DECIMAL(15, 2) NOT NULL,
    transaction_type VARCHAR(20) NOT NULL,
    category VARCHAR(100),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT amount_positive CHECK (amount > 0),
    CONSTRAINT valid_transaction_type CHECK (transaction_type IN ('income', 'expense'))
);

CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_wallet_id ON transactions(wallet_id);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(transaction_type);

CREATE OR REPLACE FUNCTION update_transactions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_transactions_updated_at ON transactions;
CREATE TRIGGER trigger_transactions_updated_at
    BEFORE UPDATE ON transactions
    FOR EACH ROW
    EXECUTE FUNCTION update_transactions_updated_at();

-- STEP 4: Create debts table
CREATE TABLE IF NOT EXISTS debts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(100) NOT NULL,
    wallet_id UUID,
    creditor_name VARCHAR(255) NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    interest_rate DECIMAL(5, 2) DEFAULT 0.00,
    due_date TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_debts_wallet_id FOREIGN KEY (wallet_id) 
        REFERENCES wallets(id) ON DELETE SET NULL,
    CONSTRAINT amount_positive CHECK (amount > 0),
    CONSTRAINT interest_rate_non_negative CHECK (interest_rate >= 0),
    CONSTRAINT valid_status CHECK (status IN ('active', 'paid', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS idx_debts_user_id ON debts(user_id);
CREATE INDEX IF NOT EXISTS idx_debts_wallet_id ON debts(wallet_id);
CREATE INDEX IF NOT EXISTS idx_debts_status ON debts(status);
CREATE INDEX IF NOT EXISTS idx_debts_due_date ON debts(due_date);

CREATE OR REPLACE FUNCTION update_debts_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_debts_updated_at ON debts;
CREATE TRIGGER trigger_debts_updated_at
    BEFORE UPDATE ON debts
    FOR EACH ROW
    EXECUTE FUNCTION update_debts_updated_at();

-- ============================================================================
-- Aggregation views for statistics
-- ============================================================================

CREATE OR REPLACE VIEW v_transaction_summary AS
SELECT 
    user_id,
    wallet_id,
    transaction_type,
    COUNT(*) as transaction_count,
    SUM(amount) as total_amount,
    AVG(amount) as average_amount,
    MIN(created_at) as first_transaction,
    MAX(created_at) as last_transaction
FROM transactions
GROUP BY user_id, wallet_id, transaction_type;

CREATE OR REPLACE VIEW v_debt_summary AS
SELECT 
    user_id,
    status,
    COUNT(*) as debt_count,
    SUM(amount) as total_debt,
    SUM(amount * (interest_rate / 100)) as estimated_annual_interest,
    COUNT(CASE WHEN due_date IS NOT NULL AND due_date < CURRENT_TIMESTAMP THEN 1 END) as overdue_count
FROM debts
GROUP BY user_id, status;

CREATE OR REPLACE VIEW v_wallet_summary AS
SELECT 
    w.id,
    w.user_id,
    w.name,
    w.wallet_type,
    w.balance,
    w.credit_limit,
    CASE 
        WHEN w.wallet_type::text = 'CreditCard' AND w.credit_limit > 0 
        THEN (w.credit_limit - w.balance)
        ELSE w.balance 
    END as available_balance,
    COUNT(t.id) as transaction_count,
    MAX(t.created_at) as last_transaction_date,
    w.created_at,
    w.updated_at
FROM wallets w
LEFT JOIN transactions t ON w.id = t.wallet_id
GROUP BY w.id, w.user_id, w.name, w.wallet_type, w.balance, w.credit_limit, w.created_at, w.updated_at;
