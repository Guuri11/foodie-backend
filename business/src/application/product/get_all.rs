use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::logger::Logger;
use crate::domain::product::errors::ProductError;
use crate::domain::product::model::Product;
use crate::domain::product::repository::ProductRepository;
use crate::domain::product::use_cases::get_all::GetAllProductsUseCase;

pub struct GetAllProductsUseCaseImpl {
    pub repository: Arc<dyn ProductRepository>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl GetAllProductsUseCase for GetAllProductsUseCaseImpl {
    async fn execute(&self) -> Result<Vec<Product>, ProductError> {
        self.logger.info("Fetching all active products");
        let products = self.repository.get_active_products().await?;
        self.logger
            .info(&format!("Found {} active products", products.len()));
        Ok(products)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::RepositoryError;
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
    async fn should_return_all_active_products_when_requested() {
        let mut mock_repo = MockProductRepo::new();
        let now = Utc::now();
        mock_repo.expect_get_active_products().returning(move || {
            Ok(vec![Product::from_repository(
                Uuid::new_v4(),
                "Tomatoes".to_string(),
                ProductStatus::New,
                None,
                Some("500g".to_string()),
                None,
                None,
                None,
                now,
                now,
            )])
        });

        let use_case = GetAllProductsUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case.execute().await;

        assert!(result.is_ok());
        let products = result.unwrap();
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].name, "Tomatoes");
    }
}
