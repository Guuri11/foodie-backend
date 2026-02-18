use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::logger::Logger;
use crate::domain::shopping_item::errors::ShoppingItemError;
use crate::domain::shopping_item::model::ShoppingItem;
use crate::domain::shopping_item::repository::ShoppingItemRepository;
use crate::domain::shopping_item::use_cases::create::{
    CreateShoppingItemParams, CreateShoppingItemUseCase,
};

pub struct CreateShoppingItemUseCaseImpl {
    pub repository: Arc<dyn ShoppingItemRepository>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl CreateShoppingItemUseCase for CreateShoppingItemUseCaseImpl {
    async fn execute(
        &self,
        params: CreateShoppingItemParams,
    ) -> Result<ShoppingItem, ShoppingItemError> {
        self.logger
            .info(&format!("Creating shopping item: {}", params.name));

        // If product_id is provided, check if it already exists in the list (skip silently)
        if let Some(product_id) = params.product_id
            && let Ok(Some(_)) = self
                .repository
                .find_by_product_id(product_id, &params.user_id)
                .await
        {
            self.logger.info(&format!(
                "Shopping item for product {} already exists, skipping",
                product_id
            ));
            let existing = self
                .repository
                .find_by_product_id(product_id, &params.user_id)
                .await?
                .ok_or(ShoppingItemError::NotFound)?;
            return Ok(existing);
        }

        let item = ShoppingItem::new(params.user_id, params.name, params.product_id)?;
        self.repository.save(&item).await?;

        self.logger
            .info(&format!("Shopping item created: {}", item.id));
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::RepositoryError;
    use crate::domain::shared::value_objects::UserId;
    use mockall::mock;
    use uuid::Uuid;

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

    #[tokio::test]
    async fn should_create_shopping_item_when_valid() {
        let mut mock_repo = MockShoppingItemRepo::new();
        mock_repo.expect_save().returning(|_| Ok(()));

        let use_case = CreateShoppingItemUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(CreateShoppingItemParams {
                user_id: test_user_id(),
                name: "Extra Virgin Olive Oil".to_string(),
                product_id: None,
            })
            .await;

        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.name, "Extra Virgin Olive Oil");
        assert!(!item.is_bought);
    }

    #[tokio::test]
    async fn should_reject_when_name_empty() {
        let mock_repo = MockShoppingItemRepo::new();

        let use_case = CreateShoppingItemUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(CreateShoppingItemParams {
                user_id: test_user_id(),
                name: "".to_string(),
                product_id: None,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ShoppingItemError::NameEmpty));
    }

    #[tokio::test]
    async fn should_not_duplicate_when_product_id_already_in_list() {
        let product_id = Uuid::new_v4();
        let existing_item = ShoppingItem::from_repository(
            Uuid::new_v4(),
            test_user_id(),
            "Milk".to_string(),
            Some(product_id),
            false,
            chrono::Utc::now(),
            chrono::Utc::now(),
        );

        let mut mock_repo = MockShoppingItemRepo::new();
        let existing_clone = existing_item.clone();
        mock_repo
            .expect_find_by_product_id()
            .returning(move |_, _| Ok(Some(existing_clone.clone())));

        let use_case = CreateShoppingItemUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(CreateShoppingItemParams {
                user_id: test_user_id(),
                name: "Milk".to_string(),
                product_id: Some(product_id),
            })
            .await;

        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.id, existing_item.id);
    }

    #[tokio::test]
    async fn should_create_manual_item_without_product_id() {
        let mut mock_repo = MockShoppingItemRepo::new();
        mock_repo.expect_save().returning(|_| Ok(()));

        let use_case = CreateShoppingItemUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(CreateShoppingItemParams {
                user_id: test_user_id(),
                name: "Bread".to_string(),
                product_id: None,
            })
            .await;

        assert!(result.is_ok());
        let item = result.unwrap();
        assert!(item.product_id.is_none());
    }
}
