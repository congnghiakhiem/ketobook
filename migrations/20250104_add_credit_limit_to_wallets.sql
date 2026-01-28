-- Add credit_limit field to wallets table for credit card support
ALTER TABLE wallets
ADD COLUMN credit_limit DECIMAL(15, 2) DEFAULT 0.00,
ADD CONSTRAINT credit_limit_non_negative CHECK (credit_limit >= 0);

-- Create index for credit card queries
CREATE INDEX IF NOT EXISTS idx_wallets_credit_card ON wallets(user_id, wallet_type) WHERE wallet_type = 'CreditCard';

-- Add comment for clarity
COMMENT ON COLUMN wallets.credit_limit IS 'Credit limit for CreditCard wallets. For other types, this remains 0.';
COMMENT ON COLUMN wallets.balance IS 'For CreditCard: available credit remaining. For others: current balance.';
