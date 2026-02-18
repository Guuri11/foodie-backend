use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::logger::Logger;
use crate::domain::product::repository::ProductRepository;
use crate::domain::product::urgency::{UrgencyLevel, get_urgency_level, is_expired};
use crate::domain::suggestion::errors::SuggestionError;
use crate::domain::suggestion::model::Suggestion;
use crate::domain::suggestion::services::SuggestionGeneratorService;
use crate::domain::suggestion::use_cases::generate::{
    GenerateSuggestionsParams, GenerateSuggestionsUseCase,
};

pub struct GenerateSuggestionsUseCaseImpl {
    pub repository: Arc<dyn ProductRepository>,
    pub generator: Arc<dyn SuggestionGeneratorService>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl GenerateSuggestionsUseCase for GenerateSuggestionsUseCaseImpl {
    async fn execute(
        &self,
        params: GenerateSuggestionsParams,
    ) -> Result<Vec<Suggestion>, SuggestionError> {
        self.logger.info(&format!(
            "Generating suggestions with limit: {}",
            params.limit
        ));

        let products = self
            .repository
            .get_active_products(&params.user_id)
            .await
            .map_err(|_| SuggestionError::GenerationFailed)?;

        // Filter out expired products
        let mut usable: Vec<_> = products.into_iter().filter(|p| !is_expired(p)).collect();

        if usable.is_empty() {
            return Ok(vec![]);
        }

        // Sort by urgency: most urgent first
        usable.sort_by(|a, b| {
            let urgency_order = |level: &UrgencyLevel| -> u8 {
                match level {
                    UrgencyLevel::UseToday => 0,
                    UrgencyLevel::UseSoon => 1,
                    UrgencyLevel::Ok => 2,
                    UrgencyLevel::WouldntTrust => 3,
                }
            };
            let a_urgency = urgency_order(&get_urgency_level(a));
            let b_urgency = urgency_order(&get_urgency_level(b));
            a_urgency.cmp(&b_urgency)
        });

        let suggestions = self.generator.generate(&usable, params.limit).await?;

        self.logger
            .info(&format!("Generated {} suggestions", suggestions.len()));

        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::RepositoryError;
    use crate::domain::product::model::Product;
    use crate::domain::product::value_objects::ProductStatus;
    use crate::domain::shared::value_objects::UserId;
    use crate::domain::suggestion::model::{Suggestion, SuggestionIngredient, TimeRange};
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
        pub SuggestionGenerator {}

        #[async_trait]
        impl SuggestionGeneratorService for SuggestionGenerator {
            async fn generate(
                &self,
                products: &[Product],
                limit: usize,
            ) -> Result<Vec<Suggestion>, SuggestionError>;
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

    fn product_expiring_in(name: &str, days: i64) -> Product {
        Product::from_repository(
            Uuid::new_v4(),
            test_user_id(),
            name.to_string(),
            ProductStatus::Opened,
            None,
            None,
            Some(Utc::now() + Duration::days(days)),
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn expired_product(name: &str) -> Product {
        Product::from_repository(
            Uuid::new_v4(),
            test_user_id(),
            name.to_string(),
            ProductStatus::Opened,
            None,
            None,
            Some(Utc::now() - Duration::days(2)),
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn sample_suggestion() -> Suggestion {
        Suggestion {
            id: "test-1".to_string(),
            title: "Pasta con pollo".to_string(),
            description: Some("Quick pasta dish".to_string()),
            estimated_time: TimeRange::Quick,
            ingredients: vec![SuggestionIngredient {
                product_id: "p1".to_string(),
                product_name: "Chicken".to_string(),
                quantity: None,
                is_urgent: true,
            }],
            urgent_ingredients: vec!["p1".to_string()],
            steps: Some(vec!["Cook pasta".to_string(), "Add chicken".to_string()]),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn should_return_suggestions_when_products_available() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_get_active_products().returning(|_| {
            Ok(vec![
                product_expiring_in("Chicken breast", 1),
                product_expiring_in("Rice", 30),
            ])
        });

        let mut mock_generator = MockSuggestionGenerator::new();
        mock_generator
            .expect_generate()
            .returning(|_, _| Ok(vec![sample_suggestion()]));

        let use_case = GenerateSuggestionsUseCaseImpl {
            repository: Arc::new(mock_repo),
            generator: Arc::new(mock_generator),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(GenerateSuggestionsParams {
                user_id: test_user_id(),
                limit: 5,
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn should_return_empty_when_no_active_products() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo
            .expect_get_active_products()
            .returning(|_| Ok(vec![]));

        let mock_generator = MockSuggestionGenerator::new();

        let use_case = GenerateSuggestionsUseCaseImpl {
            repository: Arc::new(mock_repo),
            generator: Arc::new(mock_generator),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(GenerateSuggestionsParams {
                user_id: test_user_id(),
                limit: 5,
            })
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn should_filter_out_expired_products_before_generating() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_get_active_products().returning(|_| {
            Ok(vec![
                expired_product("Old yogurt"),
                product_expiring_in("Fresh milk", 2),
            ])
        });

        let mut mock_generator = MockSuggestionGenerator::new();
        mock_generator
            .expect_generate()
            .withf(|products, _| {
                // Only the non-expired product should be passed
                products.len() == 1 && products[0].name == "Fresh milk"
            })
            .returning(|_, _| Ok(vec![sample_suggestion()]));

        let use_case = GenerateSuggestionsUseCaseImpl {
            repository: Arc::new(mock_repo),
            generator: Arc::new(mock_generator),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(GenerateSuggestionsParams {
                user_id: test_user_id(),
                limit: 5,
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_empty_when_all_products_expired() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_get_active_products().returning(|_| {
            Ok(vec![
                expired_product("Old yogurt"),
                expired_product("Expired milk"),
            ])
        });

        let mock_generator = MockSuggestionGenerator::new();

        let use_case = GenerateSuggestionsUseCaseImpl {
            repository: Arc::new(mock_repo),
            generator: Arc::new(mock_generator),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(GenerateSuggestionsParams {
                user_id: test_user_id(),
                limit: 5,
            })
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn should_return_error_when_repository_fails() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo
            .expect_get_active_products()
            .returning(|_| Err(RepositoryError::Persistence));

        let mock_generator = MockSuggestionGenerator::new();

        let use_case = GenerateSuggestionsUseCaseImpl {
            repository: Arc::new(mock_repo),
            generator: Arc::new(mock_generator),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(GenerateSuggestionsParams {
                user_id: test_user_id(),
                limit: 5,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SuggestionError::GenerationFailed
        ));
    }
}
