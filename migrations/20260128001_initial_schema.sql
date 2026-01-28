-- KetoBook Initial Schema Migration (2026-01-28)
-- Manually structured with explicit dependency ordering
-- Execution order: ENUM → wallets → transactions → debts

-- ============================================================================
-- STEP 1: Create ENUM types (no dependencies)
-- ============================================================================

CREATE TYPE IF NOT EXISTS wallet_type AS ENUM (
    'Cash',
    'BankAccount', 
    'CreditCard',
    'Other'
);

COMMENT ON TYPE wallet_type IS 'Wallet types for organizing user finances';

-- ============================================================================
-- STEP 2: Create wallets table (depends on wallet_type ENUM)
-- ============================================================================

CREATE TABLE IF NOT EXISTS wallets (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    wallet_type wallet_type NOT NULL DEFAULT 'Cash',
    balance DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    credit_limit DECIMAL(15, 2) DEFAULT 0.00,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    CONSTRAINT balance_non_negative CHECK (balance >= 0),
    CONSTRAINT credit_limit_non_negative CHECK (credit_limit >= 0),
    CONSTRAINT user_id_not_empty CHECK (user_id != '')
);

COMMENT ON TABLE wallets IS 'User wallets for managing different account types (Cash, Bank, Credit Card, Other)';
COMMENT ON COLUMN wallets.id IS 'Unique wallet identifier (UUID)';
COMMENT ON COLUMN wallets.user_id IS 'Reference to the user who owns this wallet';
COMMENT ON COLUMN wallets.name IS 'Display name for the wallet (e.g., "My Checking Account")';
COMMENT ON COLUMN wallets.wallet_type IS 'Type of wallet: Cash, BankAccount, CreditCard, or Other';
COMMENT ON COLUMN wallets.balance IS 'Current balance in the wallet. For CreditCard: represents current debt (0=no debt, limit=fully used)';
COMMENT ON COLUMN wallets.credit_limit IS 'Credit limit for CreditCard wallets. Remains 0.00 for other wallet types. Available credit = credit_limit - balance';
COMMENT ON COLUMN wallets.created_at IS 'Timestamp when wallet was created (UTC 2026)';
COMMENT ON COLUMN wallets.updated_at IS 'Timestamp when wallet was last updated (UTC 2026)';

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_wallets_wallet_type ON wallets(wallet_type);
CREATE INDEX IF NOT EXISTS idx_wallets_created_at ON wallets(created_at);
CREATE INDEX IF NOT EXISTS idx_wallets_credit_card 
    ON wallets(id) WHERE wallet_type = 'CreditCard';

-- Create trigger to auto-update updated_at
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

-- ============================================================================
-- STEP 3: Create transactions table (depends on wallets via FK)
-- ============================================================================

CREATE TABLE IF NOT EXISTS transactions (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(100) NOT NULL,
    wallet_id VARCHAR(36) NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    transaction_type VARCHAR(20) NOT NULL,
    category VARCHAR(100),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign Key constraint (CASCADE DELETE when wallet is deleted)
    CONSTRAINT fk_transactions_wallet_id FOREIGN KEY (wallet_id) 
        REFERENCES wallets(id) ON DELETE CASCADE,
    
    -- Data integrity constraints
    CONSTRAINT amount_positive CHECK (amount > 0),
    CONSTRAINT valid_transaction_type CHECK (transaction_type IN ('income', 'expense')),
    CONSTRAINT user_id_not_empty CHECK (user_id != '')
);

COMMENT ON TABLE transactions IS 'Financial transactions recorded against wallet accounts';
COMMENT ON COLUMN transactions.id IS 'Unique transaction identifier (UUID)';
COMMENT ON COLUMN transactions.user_id IS 'Reference to the user who performed the transaction';
COMMENT ON COLUMN transactions.wallet_id IS 'Reference to the wallet this transaction affects (CASCADE DELETE)';
COMMENT ON COLUMN transactions.amount IS 'Transaction amount in currency units (DECIMAL 15,2 for precision)';
COMMENT ON COLUMN transactions.transaction_type IS 'Type of transaction: income or expense';
COMMENT ON COLUMN transactions.category IS 'Category classification (e.g., groceries, salary, utilities)';
COMMENT ON COLUMN transactions.description IS 'Additional details about the transaction';
COMMENT ON COLUMN transactions.created_at IS 'Timestamp when transaction was created (UTC 2026)';
COMMENT ON COLUMN transactions.updated_at IS 'Timestamp when transaction was last updated (UTC 2026)';

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_wallet_id ON transactions(wallet_id);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_user_wallet 
    ON transactions(user_id, wallet_id);
CREATE INDEX IF NOT EXISTS idx_transactions_user_created 
    ON transactions(user_id, created_at DESC);

-- Create trigger to auto-update updated_at
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

-- ============================================================================
-- STEP 4: Create debts table (depends on wallets via FK)
-- ============================================================================

CREATE TABLE IF NOT EXISTS debts (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(100) NOT NULL,
    wallet_id VARCHAR(36),
    creditor_name VARCHAR(255) NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    interest_rate DECIMAL(5, 2) DEFAULT 0.00,
    due_date TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign Key constraint to wallet (if debt is tied to a specific wallet)
    CONSTRAINT fk_debts_wallet_id FOREIGN KEY (wallet_id) 
        REFERENCES wallets(id) ON DELETE SET NULL,
    
    -- Data integrity constraints
    CONSTRAINT amount_positive CHECK (amount > 0),
    CONSTRAINT interest_rate_non_negative CHECK (interest_rate >= 0),
    CONSTRAINT valid_status CHECK (status IN ('active', 'paid', 'cancelled')),
    CONSTRAINT user_id_not_empty CHECK (user_id != '')
);

COMMENT ON TABLE debts IS 'Debt tracking for loans, credits, and obligations';
COMMENT ON COLUMN debts.id IS 'Unique debt identifier (UUID)';
COMMENT ON COLUMN debts.user_id IS 'Reference to the user who owes the debt';
COMMENT ON COLUMN debts.wallet_id IS 'Optional reference to the wallet responsible for this debt (ON DELETE SET NULL)';
COMMENT ON COLUMN debts.creditor_name IS 'Name of the creditor (bank, person, company, etc.)';
COMMENT ON COLUMN debts.amount IS 'Principal debt amount in currency units (DECIMAL 15,2 for precision)';
COMMENT ON COLUMN debts.interest_rate IS 'Annual interest rate as a percentage (DECIMAL 5,2)';
COMMENT ON COLUMN debts.due_date IS 'Optional payment due date (UTC 2026)';
COMMENT ON COLUMN debts.status IS 'Debt status: active, paid, or cancelled';
COMMENT ON COLUMN debts.created_at IS 'Timestamp when debt was recorded (UTC 2026)';
COMMENT ON COLUMN debts.updated_at IS 'Timestamp when debt was last updated (UTC 2026)';

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_debts_user_id ON debts(user_id);
CREATE INDEX IF NOT EXISTS idx_debts_wallet_id ON debts(wallet_id);
CREATE INDEX IF NOT EXISTS idx_debts_status ON debts(status);
CREATE INDEX IF NOT EXISTS idx_debts_due_date ON debts(due_date);
CREATE INDEX IF NOT EXISTS idx_debts_user_status 
    ON debts(user_id, status);
CREATE INDEX IF NOT EXISTS idx_debts_user_due 
    ON debts(user_id, due_date) WHERE due_date IS NOT NULL;

-- Create trigger to auto-update updated_at
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
-- Optional: Create aggregation views for statistics
-- ============================================================================

-- Transaction summary view
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

COMMENT ON VIEW v_transaction_summary IS 'Aggregated transaction statistics by user, wallet, and type';

-- Debt summary view
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

COMMENT ON VIEW v_debt_summary IS 'Aggregated debt statistics by user and status';

-- Wallet summary view with calculated available credit
CREATE OR REPLACE VIEW v_wallet_summary AS
SELECT 
    w.id,
    w.user_id,
    w.name,
    w.wallet_type,
    w.balance,
    w.credit_limit,
    CASE 
        WHEN w.wallet_type = 'CreditCard' AND w.credit_limit > 0 
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

COMMENT ON VIEW v_wallet_summary IS 'Wallet summary with calculated available balance';
