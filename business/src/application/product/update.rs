use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::errors::RepositoryError;
use crate::domain::logger::Logger;
use crate::domain::product::errors::ProductError;
use crate::domain::product::model::Product;
use crate::domain::product::repository::ProductRepository;
use crate::domain::product::use_cases::update::{UpdateProductParams, UpdateProductUseCase};
use crate::domain::product::value_objects::ProductStatus;
use crate::domain::shopping_item::model::ShoppingItem;
use crate::domain::shopping_item::repository::ShoppingItemRepository;

pub struct UpdateProductUseCaseImpl {
    pub repository: Arc<dyn ProductRepository>,
    pub shopping_item_repository: Arc<dyn ShoppingItemRepository>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl UpdateProductUseCase for UpdateProductUseCaseImpl {
    async fn execute(&self, params: UpdateProductParams) -> Result<Product, ProductError> {
        self.logger
            .info(&format!("Updating product: {}", params.id));

        if params.name.trim().is_empty() {
            return Err(ProductError::NameEmpty);
        }

        if params.outcome.is_some() && params.status != ProductStatus::Finished {
            return Err(ProductError::OutcomeRequiresFinishedStatus);
        }

        // Verify product exists
        let existing = self
            .repository
            .get_by_id(params.id, &params.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => ProductError::NotFound,
                other => ProductError::Repository(other),
            })?;

        let old_status = existing.status.clone();
        let new_status = params.status.clone();

        let updated_product = Product::from_repository(
            existing.id,
            existing.user_id.clone(),
            params.name.clone(),
            params.status,
            params.location,
            params.quantity,
            params.expiry_date,
            params.estimated_expiry_date,
            params.outcome,
            existing.created_at,
            chrono::Utc::now(),
        );

        self.repository.save(&updated_product).await?;

        // Auto-add to shopping list when transitioning to Finished
        if new_status == ProductStatus::Finished
            && old_status != ProductStatus::Finished
            && let Ok(None) = self
                .shopping_item_repository
                .find_by_product_id(existing.id, &params.user_id)
                .await
            && let Ok(item) =
                ShoppingItem::new(params.user_id.clone(), params.name, Some(existing.id))
            && let Err(e) = self.shopping_item_repository.save(&item).await
        {
            self.logger.warn(&format!(
                "Failed to auto-add shopping item for product {}: {}",
                existing.id, e
            ));
        }

        // Remove from shopping list when reverting from Finished
        if old_status == ProductStatus::Finished
            && new_status != ProductStatus::Finished
            && let Err(e) = self
                .shopping_item_repository
                .delete_by_product_id(existing.id, &params.user_id)
                .await
        {
            self.logger.warn(&format!(
                "Failed to remove shopping item for product {}: {}",
                existing.id, e
            ));
        }

        self.logger
            .info(&format!("Product updated: {}", updated_product.id));
        Ok(updated_product)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::product::value_objects::{ProductOutcome, ProductStatus};
    use crate::domain::shared::value_objects::UserId;
    use crate::domain::shopping_item::model::ShoppingItem;
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
        pub ShoppingItemRepo {}

        #[async_trait]
        impl ShoppingItemRepository for ShoppingItemRepo {
            async fn get_all(&self, user_id: &UserId) -> Result<Vec<ShoppingItem>, RepositoryError>;
            async fn get_by_id(&self, id: Uuid, user_id: &UserId) -> Result<ShoppingItem, RepositoryError>;
            async fn find_by_product_id(&self, product_id: Uuid, user_id: &UserId) -> Result<Option<ShoppingItem>, RepositoryError>;
            async fn save(&self, item: &ShoppingItem) -> Result<(), RepositoryError>;
            async fn delete(&self, id: Uuid, user_id: &UserId) -> Result<(), RepositoryError>;
            async fn delete_by_product_id(&self, product_id: Uuid, user_id: &UserId) -> Result<(), RepositoryError>;
            async fn delete_bought(&self, user_id: &UserId) -> Result<u64, RepositoryError>;
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

    fn make_product(id: Uuid, status: ProductStatus) -> Product {
        Product::from_repository(
            id,
            UserId::new("test-user-id"),
            "Test Product".to_string(),
            status,
            None,
            None,
            None,
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn should_update_product_when_exists() {
        let product_id = Uuid::new_v4();
        let now = Utc::now();
        let mut mock_repo = MockProductRepo::new();
        let mut mock_shopping_repo = MockShoppingItemRepo::new();

        mock_repo.expect_get_by_id().returning(move |_, _| {
            Ok(Product::from_repository(
                product_id,
                UserId::new("test-user-id"),
                "Old Name".to_string(),
                ProductStatus::New,
                None,
                None,
                None,
                None,
                None,
                now,
                now,
            ))
        });
        mock_repo.expect_save().returning(|_| Ok(()));
        mock_shopping_repo.expect_find_by_product_id().never();

        let use_case = UpdateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            shopping_item_repository: Arc::new(mock_shopping_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateProductParams {
                id: product_id,
                user_id: test_user_id(),
                name: "Updated Olive Oil".to_string(),
                status: ProductStatus::Opened,
                location: None,
                quantity: Some("750ml".to_string()),
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: None,
            })
            .await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert_eq!(product.name, "Updated Olive Oil");
        assert_eq!(product.status, ProductStatus::Opened);
    }

    #[tokio::test]
    async fn should_reject_update_when_name_is_empty() {
        let mock_repo = MockProductRepo::new();
        let mock_shopping_repo = MockShoppingItemRepo::new();

        let use_case = UpdateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            shopping_item_repository: Arc::new(mock_shopping_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateProductParams {
                id: Uuid::new_v4(),
                user_id: test_user_id(),
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
    async fn should_reject_update_outcome_when_status_not_finished() {
        let mock_repo = MockProductRepo::new();
        let mock_shopping_repo = MockShoppingItemRepo::new();

        let use_case = UpdateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            shopping_item_repository: Arc::new(mock_shopping_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateProductParams {
                id: Uuid::new_v4(),
                user_id: test_user_id(),
                name: "Milk".to_string(),
                status: ProductStatus::Opened,
                location: None,
                quantity: None,
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: Some(ProductOutcome::ThrownAway),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProductError::OutcomeRequiresFinishedStatus
        ));
    }

    #[tokio::test]
    async fn should_return_not_found_when_updating_nonexistent_product() {
        let mut mock_repo = MockProductRepo::new();
        let mock_shopping_repo = MockShoppingItemRepo::new();
        mock_repo
            .expect_get_by_id()
            .returning(|_, _| Err(RepositoryError::NotFound));

        let use_case = UpdateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            shopping_item_repository: Arc::new(mock_shopping_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateProductParams {
                id: Uuid::new_v4(),
                user_id: test_user_id(),
                name: "Something".to_string(),
                status: ProductStatus::New,
                location: None,
                quantity: None,
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: None,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProductError::NotFound));
    }

    #[tokio::test]
    async fn should_return_not_found_when_updating_product_from_other_user() {
        let mut mock_repo = MockProductRepo::new();
        let mock_shopping_repo = MockShoppingItemRepo::new();
        // Repository returns NotFound for products belonging to other users
        mock_repo
            .expect_get_by_id()
            .returning(|_, _| Err(RepositoryError::NotFound));

        let use_case = UpdateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            shopping_item_repository: Arc::new(mock_shopping_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateProductParams {
                id: Uuid::new_v4(),
                user_id: UserId::new("other-user-id"),
                name: "Something".to_string(),
                status: ProductStatus::New,
                location: None,
                quantity: None,
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: None,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProductError::NotFound));
    }

    #[tokio::test]
    async fn should_auto_add_shopping_item_when_product_transitions_to_finished() {
        let product_id = Uuid::new_v4();
        let mut mock_repo = MockProductRepo::new();
        let mut mock_shopping_repo = MockShoppingItemRepo::new();

        mock_repo
            .expect_get_by_id()
            .returning(move |_, _| Ok(make_product(product_id, ProductStatus::Opened)));
        mock_repo.expect_save().returning(|_| Ok(()));

        mock_shopping_repo
            .expect_find_by_product_id()
            .returning(|_, _| Ok(None));
        mock_shopping_repo.expect_save().returning(|_| Ok(()));

        let use_case = UpdateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            shopping_item_repository: Arc::new(mock_shopping_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateProductParams {
                id: product_id,
                user_id: test_user_id(),
                name: "Test Product".to_string(),
                status: ProductStatus::Finished,
                location: None,
                quantity: None,
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: Some(ProductOutcome::Used),
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_not_duplicate_when_already_in_shopping_list() {
        let product_id = Uuid::new_v4();
        let mut mock_repo = MockProductRepo::new();
        let mut mock_shopping_repo = MockShoppingItemRepo::new();

        mock_repo
            .expect_get_by_id()
            .returning(move |_, _| Ok(make_product(product_id, ProductStatus::Opened)));
        mock_repo.expect_save().returning(|_| Ok(()));

        // Already exists in shopping list
        mock_shopping_repo
            .expect_find_by_product_id()
            .returning(move |_, _| {
                Ok(Some(ShoppingItem::from_repository(
                    Uuid::new_v4(),
                    UserId::new("test-user-id"),
                    "Test Product".to_string(),
                    Some(product_id),
                    false,
                    Utc::now(),
                    Utc::now(),
                )))
            });
        // save should NOT be called on shopping_item_repository
        mock_shopping_repo.expect_save().never();

        let use_case = UpdateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            shopping_item_repository: Arc::new(mock_shopping_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateProductParams {
                id: product_id,
                user_id: test_user_id(),
                name: "Test Product".to_string(),
                status: ProductStatus::Finished,
                location: None,
                quantity: None,
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: Some(ProductOutcome::Used),
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_remove_shopping_item_when_reverted_from_finished() {
        let product_id = Uuid::new_v4();
        let mut mock_repo = MockProductRepo::new();
        let mut mock_shopping_repo = MockShoppingItemRepo::new();

        mock_repo
            .expect_get_by_id()
            .returning(move |_, _| Ok(make_product(product_id, ProductStatus::Finished)));
        mock_repo.expect_save().returning(|_| Ok(()));

        mock_shopping_repo
            .expect_delete_by_product_id()
            .returning(|_, _| Ok(()));

        let use_case = UpdateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            shopping_item_repository: Arc::new(mock_shopping_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateProductParams {
                id: product_id,
                user_id: test_user_id(),
                name: "Test Product".to_string(),
                status: ProductStatus::Opened,
                location: None,
                quantity: None,
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: None,
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_not_affect_shopping_list_when_status_stays_finished() {
        let product_id = Uuid::new_v4();
        let mut mock_repo = MockProductRepo::new();
        let mut mock_shopping_repo = MockShoppingItemRepo::new();

        mock_repo
            .expect_get_by_id()
            .returning(move |_, _| Ok(make_product(product_id, ProductStatus::Finished)));
        mock_repo.expect_save().returning(|_| Ok(()));

        // Neither find_by_product_id nor delete_by_product_id should be called
        mock_shopping_repo.expect_find_by_product_id().never();
        mock_shopping_repo.expect_delete_by_product_id().never();
        mock_shopping_repo.expect_save().never();

        let use_case = UpdateProductUseCaseImpl {
            repository: Arc::new(mock_repo),
            shopping_item_repository: Arc::new(mock_shopping_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateProductParams {
                id: product_id,
                user_id: test_user_id(),
                name: "Test Product".to_string(),
                status: ProductStatus::Finished,
                location: None,
                quantity: None,
                expiry_date: None,
                estimated_expiry_date: None,
                outcome: Some(ProductOutcome::Used),
            })
            .await;

        assert!(result.is_ok());
    }
}
