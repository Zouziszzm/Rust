CREATE TABLE IF NOT EXISTS expenses (
    id UUID PRIMARY KEY,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    shop_id UUID REFERENCES shops(id) ON DELETE SET NULL,
    amount_total BIGINT NOT NULL CHECK (amount_total > 0),
    subtotal_10_percent BIGINT NOT NULL DEFAULT 0,
    tax_amount_10_percent BIGINT NOT NULL DEFAULT 0,
    subtotal_8_percent BIGINT NOT NULL DEFAULT 0,
    tax_amount_8_percent BIGINT NOT NULL DEFAULT 0,
    tax_exempt_amount BIGINT NOT NULL DEFAULT 0,
    tax_amount_total BIGINT NOT NULL DEFAULT 0,
    merchant_name VARCHAR(200),
    invoice_registration_number VARCHAR(14),
    is_qualified_invoice BOOLEAN NOT NULL DEFAULT FALSE,
    receipt_number VARCHAR(100),
    title VARCHAR(200) NOT NULL,
    notes TEXT,
    occurred_at TIMESTAMPTZ NOT NULL,
    payment_method VARCHAR(30) NOT NULL DEFAULT 'other',
    is_recurring BOOLEAN NOT NULL DEFAULT FALSE,
    is_refundable BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_expenses_occurred_at ON expenses (occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_expenses_category_id ON expenses (category_id);
CREATE INDEX IF NOT EXISTS idx_expenses_shop_id ON expenses (shop_id);
CREATE INDEX IF NOT EXISTS idx_expenses_category_occurred ON expenses (category_id, occurred_at DESC);
