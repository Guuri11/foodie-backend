use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use business::domain::errors::RepositoryError;
use business::domain::product::model::Product;
use business::domain::product::repository::ProductRepository;

use super::entity::ProductEntity;

pub struct ProductRepositoryPostgres {
    pool: PgPool,
}

impl ProductRepositoryPostgres {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for ProductRepositoryPostgres {
    async fn get_all(&self) -> Result<Vec<Product>, RepositoryError> {
        let entities = sqlx::query_as::<_, ProductEntity>(
            "SELECT id, name, status, location, quantity, expiry_date, estimated_expiry_date, outcome, created_at, updated_at FROM products ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RepositoryError::DatabaseError)?;

        Ok(entities.into_iter().map(|e| e.into_domain()).collect())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Product, RepositoryError> {
        let entity = sqlx::query_as::<_, ProductEntity>(
            "SELECT id, name, status, location, quantity, expiry_date, estimated_expiry_date, outcome, created_at, updated_at FROM products WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| RepositoryError::DatabaseError)?
        .ok_or(RepositoryError::NotFound)?;

        Ok(entity.into_domain())
    }

    async fn save(&self, product: &Product) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"INSERT INTO products (id, name, status, location, quantity, expiry_date, estimated_expiry_date, outcome, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                status = EXCLUDED.status,
                location = EXCLUDED.location,
                quantity = EXCLUDED.quantity,
                expiry_date = EXCLUDED.expiry_date,
                estimated_expiry_date = EXCLUDED.estimated_expiry_date,
                outcome = EXCLUDED.outcome,
                updated_at = EXCLUDED.updated_at"#,
        )
        .bind(product.id)
        .bind(&product.name)
        .bind(product.status.to_string())
        .bind(product.location.as_ref().map(|l| l.to_string()))
        .bind(&product.quantity)
        .bind(product.expiry_date)
        .bind(product.estimated_expiry_date)
        .bind(product.outcome.as_ref().map(|o| o.to_string()))
        .bind(product.created_at)
        .bind(product.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|_| RepositoryError::DatabaseError)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|_| RepositoryError::DatabaseError)?;

        Ok(())
    }

    async fn get_active_products(&self) -> Result<Vec<Product>, RepositoryError> {
        let entities = sqlx::query_as::<_, ProductEntity>(
            "SELECT id, name, status, location, quantity, expiry_date, estimated_expiry_date, outcome, created_at, updated_at FROM products WHERE status != 'finished' ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RepositoryError::DatabaseError)?;

        Ok(entities.into_iter().map(|e| e.into_domain()).collect())
    }
}
