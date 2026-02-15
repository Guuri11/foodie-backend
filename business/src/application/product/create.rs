use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::logger::Logger;
use crate::domain::product::errors::ProductError;
use crate::domain::product::model::{NewProductProps, Product};
use crate::domain::product::repository::ProductRepository;
use crate::domain::product::use_cases::create::{CreateProductParams, CreateProductUseCase};

pub struct CreateProductUseCaseImpl {
    pub repository: Arc<dyn ProductRepository>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl CreateProductUseCase for CreateProductUseCaseImpl {
    async fn execute(&self, params: CreateProductParams) -> Result<Product, ProductError> {
        self.logger
            .info(&format!("Creating product: {}", params.name));

        let product = Product::new(NewProductProps {
            name: params.name,
            status: params.status,
            location: params.location,
            quantity: params.quantity,
            expiry_date: params.expiry_date,
            estimated_expiry_date: params.estimated_expiry_date,
            outcome: params.outcome,
        })?;

        self.repository.save(&product).await?;

        self.logger
            .info(&format!("Product created with id: {}", product.id));
        Ok(product)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::RepositoryError;
    use crate::domain::product::value_objects::{ProductOutcome, ProductStatus};
    use mockall::mock;

    mock! {
        pub ProductRepo {}

        #[async_trait]
        impl ProductRepository for ProductRepo {
            async fn get_all(&self) -> Result<Vec<Product>, RepositoryError>;
            async fn get_by_id(&self, id: uuid::Uuid) -> Result<Product, RepositoryError>;
            async fn save(&self, product: &Product) -> Result<(), RepositoryError>;
            async fn delete(&self, id: uuid::Uuid) -> Result<(), RepositoryError>;
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
    async fn should_create_product_when_valid_name() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_save().returning(|_| Ok(()));

        let use_case = CreateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(CreateProductParams {
                name: "Extra Virgin Olive Oil".to_string(),
                status: ProductStatus::New,
                location: None,
                quantity: Some("1L".to_string()),
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: None,
            })
            .await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert_eq!(product.name, "Extra Virgin Olive Oil");
        assert_eq!(product.status, ProductStatus::New);
    }

    #[tokio::test]
    async fn should_reject_product_when_name_is_empty() {
        let mock_repo = MockProductRepo::new();

        let use_case = CreateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(CreateProductParams {
                name: "".to_string(),
                status: ProductStatus::New,
                location: None,
                quantity: None,
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: None,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProductError::NameEmpty));
    }

    #[tokio::test]
    async fn should_reject_outcome_when_status_not_finished() {
        let mock_repo = MockProductRepo::new();

        let use_case = CreateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(CreateProductParams {
                name: "Milk".to_string(),
                status: ProductStatus::New,
                location: None,
                quantity: None,
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: Some(ProductOutcome::Used),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProductError::OutcomeRequiresFinishedStatus
        ));
    }
}
