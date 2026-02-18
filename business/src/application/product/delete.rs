use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::errors::RepositoryError;
use crate::domain::logger::Logger;
use crate::domain::product::errors::ProductError;
use crate::domain::product::repository::ProductRepository;
use crate::domain::product::use_cases::delete::{DeleteProductParams, DeleteProductUseCase};

pub struct DeleteProductUseCaseImpl {
    pub repository: Arc<dyn ProductRepository>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl DeleteProductUseCase for DeleteProductUseCaseImpl {
    async fn execute(&self, params: DeleteProductParams) -> Result<(), ProductError> {
        self.logger
            .info(&format!("Deleting product: {}", params.id));

        // Verify product exists before deleting
        self.repository
            .get_by_id(params.id, &params.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => ProductError::NotFound,
                other => ProductError::Repository(other),
            })?;

        self.repository.delete(params.id, &params.user_id).await?;

        self.logger.info(&format!("Product deleted: {}", params.id));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::product::model::Product;
    use crate::domain::product::value_objects::ProductStatus;
    use crate::domain::shared::value_objects::UserId;
    use chrono::Utc;
    use mockall::mock;
    use uuid::Uuid;

    mock! {
        pub ProductRepo {}

        #[async_trait]
        impl ProductRepository for ProductRepo {
            async fn get_all(&self, user_id: &UserId) -> Result<Vec<Product>, RepositoryError>;
            async fn get_by_id(&self, id: Uuid, user_id: &UserId) -> Result<Product, RepositoryError>;
            async fn save(&self, product: &Product) -> Result<(), RepositoryError>;
            async fn delete(&self, id: Uuid, user_id: &UserId) -> Result<(), RepositoryError>;
            async fn get_active_products(&self, user_id: &UserId) -> Result<Vec<Product>, RepositoryError>;
        }
    }

    mock! {
        pub Log {}

        impl Logger for Log {
            fn info(&self, message: &str);
            fn warn(&self, message: &str);
            fn error(&self, message: &str);
            fn debug(&self, message: &str);
        }
    }

    fn test_user_id() -> UserId {
        UserId::new("test-user-id")
    }

    fn mock_logger() -> Arc<dyn Logger> {
        let mut logger = MockLog::new();
        logger.expect_info().returning(|_| ());
        logger.expect_warn().returning(|_| ());
        logger.expect_error().returning(|_| ());
        logger.expect_debug().returning(|_| ());
        Arc::new(logger)
    }

    #[tokio::test]
    async fn should_delete_product_when_exists() {
        let product_id = Uuid::new_v4();
        let now = Utc::now();
        let mut mock_repo = MockProductRepo::new();

        mock_repo.expect_get_by_id().returning(move |_, _| {
            Ok(Product::from_repository(
                product_id,
                UserId::new("test-user-id"),
                "Expired Yogurt".to_string(),
                ProductStatus::Finished,
                None,
                None,
                None,
                None,
                None,
                now,
                now,
            ))
        });
        mock_repo.expect_delete().returning(|_, _| Ok(()));

        let use_case = DeleteProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(DeleteProductParams {
                id: product_id,
                user_id: test_user_id(),
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_not_found_when_deleting_nonexistent_product() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo
            .expect_get_by_id()
            .returning(|_, _| Err(RepositoryError::NotFound));

        let use_case = DeleteProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(DeleteProductParams {
                id: Uuid::new_v4(),
                user_id: test_user_id(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProductError::NotFound));
    }

    #[tokio::test]
    async fn should_return_not_found_when_deleting_product_from_other_user() {
        let mut mock_repo = MockProductRepo::new();
        // Repository returns NotFound for products belonging to other users
        mock_repo
            .expect_get_by_id()
            .returning(|_, _| Err(RepositoryError::NotFound));

        let use_case = DeleteProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(DeleteProductParams {
                id: Uuid::new_v4(),
                user_id: UserId::new("other-user-id"),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProductError::NotFound));
    }
}
