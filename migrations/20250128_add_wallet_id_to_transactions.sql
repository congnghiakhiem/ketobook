-- Add wallet_id foreign key to transactions table
ALTER TABLE transactions
ADD COLUMN wallet_id VARCHAR,
ADD CONSTRAINT fk_transactions_wallet_id FOREIGN KEY (wallet_id) REFERENCES wallets(id) ON DELETE CASCADE;

-- Create index for wallet_id queries
CREATE INDEX IF NOT EXISTS idx_transactions_wallet_id ON transactions(wallet_id);
CREATE INDEX IF NOT EXISTS idx_transactions_user_wallet ON transactions(user_id, wallet_id);

-- Create composite index for common queries
CREATE INDEX IF NOT EXISTS idx_transactions_wallet_created ON transactions(wallet_id, created_at DESC);
