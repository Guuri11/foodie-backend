use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::errors::RepositoryError;
use crate::domain::logger::Logger;
use crate::domain::product::errors::ProductError;
use crate::domain::product::model::Product;
use crate::domain::product::repository::ProductRepository;
use crate::domain::product::use_cases::get_by_id::{GetProductByIdParams, GetProductByIdUseCase};

pub struct GetProductByIdUseCaseImpl {
    pub repository: Arc<dyn ProductRepository>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl GetProductByIdUseCase for GetProductByIdUseCaseImpl {
    async fn execute(&self, params: GetProductByIdParams) -> Result<Product, ProductError> {
        self.logger
            .info(&format!("Fetching product by id: {}", params.id));

        let product = self
            .repository
            .get_by_id(params.id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => ProductError::NotFound,
                other => ProductError::Repository(other),
            })?;

        Ok(product)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::product::value_objects::ProductStatus;
    use chrono::Utc;
    use mockall::mock;
    use uuid::Uuid;

    mock! {
        pub ProductRepo {}

        #[async_trait]
        impl ProductRepository for ProductRepo {
            async fn get_all(&self) -> Result<Vec<Product>, RepositoryError>;
            async fn get_by_id(&self, id: Uuid) -> Result<Product, RepositoryError>;
            async fn save(&self, product: &Product) -> Result<(), RepositoryError>;
            async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
            async fn get_active_products(&self) -> Result<Vec<Product>, RepositoryError>;
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

    fn mock_logger() -> Arc<dyn Logger> {
        let mut logger = MockLog::new();
        logger.expect_info().returning(|_| ());
        logger.expect_warn().returning(|_| ());
        logger.expect_error().returning(|_| ());
        logger.expect_debug().returning(|_| ());
        Arc::new(logger)
    }

    #[tokio::test]
    async fn should_return_product_when_exists() {
        let product_id = Uuid::new_v4();
        let now = Utc::now();
        let mut mock_repo = MockProductRepo::new();

        let id_clone = product_id;
        mock_repo
            .expect_get_by_id()
            .withf(move |id| *id == id_clone)
            .returning(move |_| {
                Ok(Product::from_repository(
                    product_id,
                    "Fresh Salmon".to_string(),
                    ProductStatus::New,
                    None,
                    Some("200g".to_string()),
                    None,
                    None,
                    None,
                    now,
                    now,
                ))
            });

        let use_case = GetProductByIdUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(GetProductByIdParams { id: product_id })
            .await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert_eq!(product.id, product_id);
        assert_eq!(product.name, "Fresh Salmon");
    }

    #[tokio::test]
    async fn should_return_error_when_product_not_found() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo
            .expect_get_by_id()
            .returning(|_| Err(RepositoryError::NotFound));

        let use_case = GetProductByIdUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(GetProductByIdParams { id: Uuid::new_v4() })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProductError::NotFound));
    }
}
