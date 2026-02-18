-- Add user_id column to products and shopping_items for multi-user data isolation
-- Database is empty, so no DEFAULT needed

ALTER TABLE products ADD COLUMN user_id VARCHAR(128) NOT NULL;
CREATE INDEX idx_products_user_id ON products(user_id);

ALTER TABLE shopping_items ADD COLUMN user_id VARCHAR(128) NOT NULL;
CREATE INDEX idx_shopping_items_user_id ON shopping_items(user_id);
