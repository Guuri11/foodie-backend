use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use business::domain::errors::RepositoryError;
use business::domain::shared::value_objects::UserId;
use business::domain::shopping_item::model::ShoppingItem;
use business::domain::shopping_item::repository::ShoppingItemRepository;

use super::entity::ShoppingItemEntity;

pub struct ShoppingItemRepositoryPostgres {
    pool: PgPool,
}

impl ShoppingItemRepositoryPostgres {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShoppingItemRepository for ShoppingItemRepositoryPostgres {
    async fn get_all(&self, user_id: &UserId) -> Result<Vec<ShoppingItem>, RepositoryError> {
        let entities = sqlx::query_as::<_, ShoppingItemEntity>(
            "SELECT id, user_id, name, product_id, is_bought, created_at, updated_at FROM shopping_items WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RepositoryError::DatabaseError)?;

        Ok(entities.into_iter().map(|e| e.into_domain()).collect())
    }

    async fn get_by_id(&self, id: Uuid, user_id: &UserId) -> Result<ShoppingItem, RepositoryError> {
        let entity = sqlx::query_as::<_, ShoppingItemEntity>(
            "SELECT id, user_id, name, product_id, is_bought, created_at, updated_at FROM shopping_items WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| RepositoryError::DatabaseError)?
        .ok_or(RepositoryError::NotFound)?;

        Ok(entity.into_domain())
    }

    async fn find_by_product_id(
        &self,
        product_id: Uuid,
        user_id: &UserId,
    ) -> Result<Option<ShoppingItem>, RepositoryError> {
        let entity = sqlx::query_as::<_, ShoppingItemEntity>(
            "SELECT id, user_id, name, product_id, is_bought, created_at, updated_at FROM shopping_items WHERE product_id = $1 AND user_id = $2",
        )
        .bind(product_id)
        .bind(user_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| RepositoryError::DatabaseError)?;

        Ok(entity.map(|e| e.into_domain()))
    }

    async fn save(&self, item: &ShoppingItem) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"INSERT INTO shopping_items (id, user_id, name, product_id, is_bought, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                is_bought = EXCLUDED.is_bought,
                updated_at = EXCLUDED.updated_at"#,
        )
        .bind(item.id)
        .bind(item.user_id.as_str())
        .bind(&item.name)
        .bind(item.product_id)
        .bind(item.is_bought)
        .bind(item.created_at)
        .bind(item.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|_| RepositoryError::DatabaseError)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid, user_id: &UserId) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM shopping_items WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id.as_str())
            .execute(&self.pool)
            .await
            .map_err(|_| RepositoryError::DatabaseError)?;

        Ok(())
    }

    async fn delete_by_product_id(
        &self,
        product_id: Uuid,
        user_id: &UserId,
    ) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM shopping_items WHERE product_id = $1 AND user_id = $2")
            .bind(product_id)
            .bind(user_id.as_str())
            .execute(&self.pool)
            .await
            .map_err(|_| RepositoryError::DatabaseError)?;

        Ok(())
    }

    async fn delete_bought(&self, user_id: &UserId) -> Result<u64, RepositoryError> {
        let result =
            sqlx::query("DELETE FROM shopping_items WHERE is_bought = TRUE AND user_id = $1")
                .bind(user_id.as_str())
                .execute(&self.pool)
                .await
                .map_err(|_| RepositoryError::DatabaseError)?;

        Ok(result.rows_affected())
    }
}
