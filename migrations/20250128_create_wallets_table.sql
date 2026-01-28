-- Create wallet_type enum
CREATE TYPE wallet_type AS ENUM ('Cash', 'BankAccount', 'CreditCard', 'Other');

-- Create wallets table
CREATE TABLE IF NOT EXISTS wallets (
    id VARCHAR PRIMARY KEY,
    user_id VARCHAR NOT NULL,
    name VARCHAR(100) NOT NULL,
    balance DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    wallet_type wallet_type NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT balance_non_negative CHECK (balance >= 0)
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_wallets_user_type ON wallets(user_id, wallet_type);
CREATE INDEX IF NOT EXISTS idx_wallets_created_at ON wallets(created_at DESC);

-- Create trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_wallets_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER wallets_update_timestamp
BEFORE UPDATE ON wallets
FOR EACH ROW
EXECUTE FUNCTION update_wallets_updated_at();
