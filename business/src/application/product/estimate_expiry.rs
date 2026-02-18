use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::domain::logger::Logger;
use crate::domain::product::errors::ProductError;
use crate::domain::product::model::Product;
use crate::domain::product::repository::ProductRepository;
use crate::domain::product::services::ExpiryEstimatorService;
use crate::domain::product::use_cases::estimate_expiry::{
    EstimateExpiryParams, EstimateExpiryUseCase,
};

pub struct EstimateExpiryUseCaseImpl {
    pub repository: Arc<dyn ProductRepository>,
    pub estimator: Arc<dyn ExpiryEstimatorService>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl EstimateExpiryUseCase for EstimateExpiryUseCaseImpl {
    async fn execute(&self, params: EstimateExpiryParams) -> Result<Product, ProductError> {
        self.logger.info(&format!(
            "Estimating expiry date for product: {}",
            params.product_id
        ));

        let mut product = self
            .repository
            .get_by_id(params.product_id, &params.user_id)
            .await
            .map_err(|e| match e {
                crate::domain::errors::RepositoryError::NotFound => ProductError::NotFound,
                other => ProductError::Repository(other),
            })?;

        let status_str = product.status.to_string();
        let location_str = product.location.as_ref().map(|l| l.to_string());

        let estimation = self
            .estimator
            .estimate_expiry_date(&product.name, &status_str, location_str)
            .await;

        if let Some(date) = estimation.date {
            product.estimated_expiry_date = Some(date);
            product.updated_at = Utc::now();
            self.repository.save(&product).await?;
        }

        self.logger.info(&format!(
            "Expiry estimation complete for product {}: confidence={}",
            product.id, estimation.confidence
        ));

        Ok(product)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::RepositoryError;
    use crate::domain::product::services::{Confidence, ExpiryEstimation};
    use crate::domain::product::value_objects::ProductStatus;
    use crate::domain::shared::value_objects::UserId;
    use chrono::{Duration, Utc};
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

    fn test_user_id() -> UserId {
        UserId::new("test-user-id")
    }

    fn sample_product(id: Uuid) -> Product {
        Product::from_repository(
            id,
            test_user_id(),
            "Whole Milk".to_string(),
            ProductStatus::Opened,
            None,
            Some("1L".to_string()),
            None,
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn should_update_estimated_expiry_when_estimation_has_date() {
        let product_id = Uuid::new_v4();
        let product = sample_product(product_id);
        let estimated_date = Utc::now() + Duration::days(3);

        let mut mock_repo = MockProductRepo::new();
        mock_repo
            .expect_get_by_id()
            .withf(move |id, _| *id == product_id)
            .returning(move |_, _| Ok(product.clone()));
        mock_repo.expect_save().returning(|_| Ok(()));

        let mut mock_estimator = MockExpiryEstimator::new();
        mock_estimator
            .expect_estimate_expiry_date()
            .returning(move |_, _, _| ExpiryEstimation {
                date: Some(estimated_date),
                confidence: Confidence::High,
            });

        let use_case = EstimateExpiryUseCaseImpl {
            repository: Arc::new(mock_repo),
            estimator: Arc::new(mock_estimator),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(EstimateExpiryParams {
                product_id,
                user_id: test_user_id(),
            })
            .await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert!(updated.estimated_expiry_date.is_some());
    }

    #[tokio::test]
    async fn should_not_save_when_estimation_returns_no_date() {
        let product_id = Uuid::new_v4();
        let product = sample_product(product_id);

        let mut mock_repo = MockProductRepo::new();
        mock_repo
            .expect_get_by_id()
            .returning(move |_, _| Ok(product.clone()));
        // save should NOT be called
        mock_repo.expect_save().never();

        let mut mock_estimator = MockExpiryEstimator::new();
        mock_estimator
            .expect_estimate_expiry_date()
            .returning(|_, _, _| ExpiryEstimation {
                date: None,
                confidence: Confidence::None,
            });

        let use_case = EstimateExpiryUseCaseImpl {
            repository: Arc::new(mock_repo),
            estimator: Arc::new(mock_estimator),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(EstimateExpiryParams {
                product_id,
                user_id: test_user_id(),
            })
            .await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert!(product.estimated_expiry_date.is_none());
    }

    #[tokio::test]
    async fn should_return_not_found_when_product_does_not_exist() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo
            .expect_get_by_id()
            .returning(|_, _| Err(RepositoryError::NotFound));

        let mock_estimator = MockExpiryEstimator::new();

        let use_case = EstimateExpiryUseCaseImpl {
            repository: Arc::new(mock_repo),
            estimator: Arc::new(mock_estimator),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(EstimateExpiryParams {
                product_id: Uuid::new_v4(),
                user_id: test_user_id(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProductError::NotFound));
    }
}
