# Plan: Multi-user Data Isolation (user_id)

## Context

All endpoints require Firebase auth (implemented), but every user sees the same data. We need to add `user_id` (Firebase UID) to products and shopping items so each user only sees their own data.

## Design decisions

1. **`UserId` value object** (wrapping Firebase UID) added to domain models, use case params, and repository methods
2. **`&UserId` in parameters** — More idiomatic Rust, avoids unnecessary clones
3. **Shared value objects module** — `UserId` lives in `business/src/domain/shared/` since it's used by both product and shopping_item
4. **Stateless AI endpoints unchanged**: `identify/image`, `identify/barcode`, `scan_receipt`, `estimate-expiry-date` (the standalone one) don't access user data — no user_id needed
5. **Repository-level filtering**: All queries add `WHERE user_id = $N` — security at the DB level, not just application
6. **`FirebaseBearer.0`** is the UID — handlers extract it with `auth.0` and convert to `UserId`
7. **No background scheduler exists** in code (just env vars placeholder) — nothing to change there
8. **Migration**: Simple `ADD COLUMN NOT NULL` — database will be empty/reset

## Implementation order

### 0. Shared module for UserId value object
**New file**: `business/src/domain/shared/mod.rs`
```rust
pub mod value_objects;
```

**New file**: `business/src/domain/shared/value_objects.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
```

**Update**: `business/src/lib.rs` — add shared module

### 1. DB migration
**New file**: `infrastructure/persistence/src/migrations/<timestamp>_add_user_id.sql`
```sql
ALTER TABLE products ADD COLUMN user_id VARCHAR(128) NOT NULL;
CREATE INDEX idx_products_user_id ON products(user_id);

ALTER TABLE shopping_items ADD COLUMN user_id VARCHAR(128) NOT NULL;
CREATE INDEX idx_shopping_items_user_id ON shopping_items(user_id);
```

### 2. Domain models — add `user_id: UserId`
- **`business/src/domain/product/model.rs`**: Add `user_id: UserId` field to `Product`, `NewProductProps`, `Product::new()`, `Product::from_repository()`
- **`business/src/domain/shopping_item/model.rs`**: Add `user_id: UserId` field to `ShoppingItem`, `ShoppingItem::new()`, `ShoppingItem::from_repository()`

### 3. Repository traits — add `&UserId` parameter
- **`business/src/domain/product/repository.rs`**:
  - `get_all(&self, user_id: &UserId)`
  - `get_by_id(&self, id: Uuid, user_id: &UserId)`
  - `delete(&self, id: Uuid, user_id: &UserId)`
  - `get_active_products(&self, user_id: &UserId)`
  - `save(&self, product: &Product)` — no change, user_id is in the model
- **`business/src/domain/shopping_item/repository.rs`**:
  - `get_all(&self, user_id: &UserId)`
  - `get_by_id(&self, id: Uuid, user_id: &UserId)`
  - `find_by_product_id(&self, product_id: Uuid, user_id: &UserId)`
  - `delete(&self, id: Uuid, user_id: &UserId)`
  - `delete_by_product_id(&self, product_id: Uuid, user_id: &UserId)`
  - `delete_bought(&self, user_id: &UserId)`
  - `save(&self, item: &ShoppingItem)` — no change, user_id is in the model

### 4. Use case traits — add `user_id: UserId` to params
**Product use cases** (6 files in `business/src/domain/product/use_cases/`):
- `create.rs`: add `user_id: UserId` to `CreateProductParams`
- `get_all.rs`: add `user_id: UserId` to `GetAllProductsParams` (new struct)
- `get_by_id.rs`: add `user_id: UserId` to `GetProductByIdParams`
- `update.rs`: add `user_id: UserId` to `UpdateProductParams`
- `delete.rs`: add `user_id: UserId` to `DeleteProductParams`
- `estimate_expiry.rs`: add `user_id: UserId` to `EstimateExpiryParams`

**Shopping item use cases** (5 files in `business/src/domain/shopping_item/use_cases/`):
- `create.rs`: add `user_id: UserId` to `CreateShoppingItemParams`
- `get_all.rs`: add `user_id: UserId` to `GetAllShoppingItemsParams` (new struct)
- `update.rs`: add `user_id: UserId` to `UpdateShoppingItemParams`
- `delete.rs`: add `user_id: UserId` to `DeleteShoppingItemParams`
- `clear_bought.rs`: add `user_id: UserId` to `ClearBoughtParams` (new struct)

**Suggestion use case** (1 file in `business/src/domain/suggestion/use_cases/`):
- `generate.rs`: add `user_id: UserId` to `GenerateSuggestionsParams`

### 5. Use case implementations — pass user_id through
**12 files in `business/src/application/`**:
- `product/create.rs` — pass `user_id` into `NewProductProps`, pass to repo
- `product/get_all.rs` — pass `&user_id` to `repository.get_all(&user_id)`
- `product/get_by_id.rs` — pass `&user_id` to `repository.get_by_id(id, &user_id)`
- `product/update.rs` — pass `&user_id` to `get_by_id()`, carry in updated product. Also calls `shopping_item_repository.find_by_product_id(_, &user_id)`, `.save()`, `.delete_by_product_id(_, &user_id)`
- `product/delete.rs` — pass `&user_id` to `get_by_id()` and `delete()`
- `product/estimate_expiry.rs` — pass `&user_id` to `get_by_id()` and `save()`
- `shopping_item/create.rs` — pass `user_id` into `ShoppingItem::new()`, `find_by_product_id(_, &user_id)`
- `shopping_item/get_all.rs` — pass `&user_id` to `get_all(&user_id)`
- `shopping_item/update.rs` — pass `&user_id` to `get_by_id(_, &user_id)`
- `shopping_item/delete.rs` — pass `&user_id` to `get_by_id(_, &user_id)` and `delete(_, &user_id)`
- `shopping_item/clear_bought.rs` — pass `&user_id` to `delete_bought(&user_id)`
- `suggestion/generate.rs` — pass `&user_id` to `get_active_products(&user_id)`

### 6. Update all unit tests + NEW isolation tests
**Update ~53 existing tests** with `UserId::new("test-user-id")`

**NEW isolation tests** (one per relevant use case):
```rust
// Product tests
should_not_return_products_from_other_users_when_getting_all
should_return_not_found_when_getting_product_from_other_user
should_return_not_found_when_updating_product_from_other_user
should_return_not_found_when_deleting_product_from_other_user

// Shopping item tests
should_not_return_shopping_items_from_other_users_when_getting_all
should_return_not_found_when_updating_shopping_item_from_other_user
should_return_not_found_when_deleting_shopping_item_from_other_user
should_not_clear_bought_items_from_other_users

// Suggestion tests
should_only_suggest_from_current_user_products
```

### 7. Persistence entities — add `user_id` field
- **`infrastructure/persistence/src/product/entity.rs`**: add `user_id: String`, update conversions
- **`infrastructure/persistence/src/shopping_item/entity.rs`**: same

### 8. Persistence repositories — update SQL
- **`infrastructure/persistence/src/product/repository.rs`**: add `user_id` to all SELECT/INSERT/DELETE queries, bind params
- **`infrastructure/persistence/src/shopping_item/repository.rs`**: same pattern

### 9. Presentation routes — extract UID from auth
- **`presentation/rest-api/src/api/product/routes.rs`**: change `_auth` to `auth`, use `UserId::new(auth.0)` in params (6 endpoints: create, get_all, get_by_id, update, delete, estimate_expiry). The 4 stateless AI endpoints keep `_auth`.
- **`presentation/rest-api/src/api/shopping_item/routes.rs`**: `UserId::new(auth.0)` for all 5 endpoints
- **`presentation/rest-api/src/api/suggestion/routes.rs`**: `UserId::new(auth.0)` for the 1 endpoint

### 10. SQLx offline cache
Run `make sqlx/prepare` after all SQL changes.

## Files changed summary

| Layer | Count |
|-------|-------|
| Migration (new) | 1 |
| Shared value objects (new) | 2 |
| Domain models | 2 |
| Repository traits | 2 |
| Use case traits | 12 |
| Use case implementations | 12 |
| Persistence entities | 2 |
| Persistence repositories | 2 |
| Routes | 3 |
| **Total** | **~38** |

## Verification

```bash
make sqlx/migrate        # Apply migration
make check               # Compiles
make lint                # No warnings
make format              # Formatted
make test                # All tests pass (including new isolation tests)
make sqlx/prepare        # Update offline cache
```
