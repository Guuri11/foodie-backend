CREATE TABLE shopping_items (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    product_id UUID REFERENCES products(id) ON DELETE SET NULL,
    is_bought BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_shopping_items_product_id ON shopping_items(product_id);
CREATE INDEX idx_shopping_items_is_bought ON shopping_items(is_bought);
