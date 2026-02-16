use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::domain::logger::Logger;
use crate::domain::product::errors::ProductError;
use crate::domain::product::model::{NewProductProps, Product};
use crate::domain::product::repository::ProductRepository;
use crate::domain::product::services::ExpiryEstimatorService;
use crate::domain::product::use_cases::create::{CreateProductParams, CreateProductUseCase};

pub struct CreateProductUseCaseImpl {
    pub repository: Arc<dyn ProductRepository>,
    pub estimator: Arc<dyn ExpiryEstimatorService>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl CreateProductUseCase for CreateProductUseCaseImpl {
    async fn execute(&self, params: CreateProductParams) -> Result<Product, ProductError> {
        self.logger
            .info(&format!("Creating product: {}", params.name));

        let mut product = Product::new(NewProductProps {
            name: params.name,
            status: params.status,
            location: params.location,
            quantity: params.quantity,
            expiry_date: params.expiry_date,
            estimated_expiry_date: params.estimated_expiry_date,
            outcome: params.outcome,
        })?;

        self.repository.save(&product).await?;

        if product.expiry_date.is_none() {
            let status_str = product.status.to_string();
            let location_str = product.location.as_ref().map(|l| l.to_string());

            let estimation = self
                .estimator
                .estimate_expiry_date(&product.name, &status_str, location_str)
                .await;

            if let Some(date) = estimation.date {
                self.logger.info(&format!(
                    "Estimated expiry for product {}: confidence={}",
                    product.id, estimation.confidence
                ));
                product.estimated_expiry_date = Some(date);
                product.updated_at = Utc::now();
                self.repository.save(&product).await?;
            } else {
                self.logger.info(&format!(
                    "No expiry estimation available for product {}",
                    product.id
                ));
            }
        }

        self.logger
            .info(&format!("Product created with id: {}", product.id));
        Ok(product)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::RepositoryError;
    use crate::domain::product::services::{Confidence, ExpiryEstimation};
    use crate::domain::product::value_objects::{ProductOutcome, ProductStatus};
    use chrono::Duration;
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
        pub ExpiryEstimator {}

        #[async_trait]
        impl ExpiryEstimatorService for ExpiryEstimator {
            async fn estimate_expiry_date(
                &self,
                product_name: &str,
                status: &str,
                location: Option<String>,
            ) -> ExpiryEstimation;
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

    fn mock_estimator_returning_none() -> Arc<dyn ExpiryEstimatorService> {
        let mut estimator = MockExpiryEstimator::new();
        estimator
            .expect_estimate_expiry_date()
            .returning(|_, _, _| ExpiryEstimation {
                date: None,
                confidence: Confidence::None,
            });
        Arc::new(estimator)
    }

    #[tokio::test]
    async fn should_create_product_when_valid_name() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_save().returning(|_| Ok(()));

        let use_case = CreateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            estimator: mock_estimator_returning_none(),
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
            estimator: mock_estimator_returning_none(),
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
            estimator: mock_estimator_returning_none(),
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

    #[tokio::test]
    async fn should_estimate_expiry_when_no_expiry_date_provided() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_save().times(2).returning(|_| Ok(()));

        let estimated_date = Utc::now() + Duration::days(7);
        let mut mock_estimator = MockExpiryEstimator::new();
        mock_estimator
            .expect_estimate_expiry_date()
            .returning(move |_, _, _| ExpiryEstimation {
                date: Some(estimated_date),
                confidence: Confidence::High,
            });

        let use_case = CreateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            estimator: Arc::new(mock_estimator),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(CreateProductParams {
                name: "Fresh Salmon Fillet".to_string(),
                status: ProductStatus::New,
                location: None,
                quantity: Some("500g".to_string()),
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: None,
            })
            .await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert!(product.estimated_expiry_date.is_some());
        assert_eq!(product.estimated_expiry_date.unwrap(), estimated_date);
    }

    #[tokio::test]
    async fn should_skip_estimation_when_expiry_date_already_provided() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_save().times(1).returning(|_| Ok(()));

        let mut mock_estimator = MockExpiryEstimator::new();
        mock_estimator.expect_estimate_expiry_date().never();

        let expiry_date = Utc::now() + Duration::days(14);

        let use_case = CreateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            estimator: Arc::new(mock_estimator),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(CreateProductParams {
                name: "Greek Yogurt".to_string(),
                status: ProductStatus::New,
                location: None,
                quantity: Some("500g".to_string()),
                expiry_date: Some(expiry_date),
                estimated_expiry_date: None,
                outcome: None,
            })
            .await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert_eq!(product.expiry_date.unwrap(), expiry_date);
        assert!(product.estimated_expiry_date.is_none());
    }

    #[tokio::test]
    async fn should_create_product_even_when_estimation_fails() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_save().times(1).returning(|_| Ok(()));

        let use_case = CreateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            estimator: mock_estimator_returning_none(),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(CreateProductParams {
                name: "Artisan Sourdough Bread".to_string(),
                status: ProductStatus::New,
                location: None,
                quantity: Some("1 loaf".to_string()),
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: None,
            })
            .await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert_eq!(product.name, "Artisan Sourdough Bread");
        assert!(product.estimated_expiry_date.is_none());
    }
}
