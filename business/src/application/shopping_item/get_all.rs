use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::logger::Logger;
use crate::domain::shopping_item::errors::ShoppingItemError;
use crate::domain::shopping_item::model::ShoppingItem;
use crate::domain::shopping_item::repository::ShoppingItemRepository;
use crate::domain::shopping_item::use_cases::get_all::{
    GetAllShoppingItemsParams, GetAllShoppingItemsUseCase,
};

pub struct GetAllShoppingItemsUseCaseImpl {
    pub repository: Arc<dyn ShoppingItemRepository>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl GetAllShoppingItemsUseCase for GetAllShoppingItemsUseCaseImpl {
    async fn execute(
        &self,
        params: GetAllShoppingItemsParams,
    ) -> Result<Vec<ShoppingItem>, ShoppingItemError> {
        self.logger.info("Getting all shopping items");
        let items = self.repository.get_all(&params.user_id).await?;
        self.logger
            .info(&format!("Retrieved {} shopping items", items.len()));
        Ok(items)
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
    async fn should_return_all_shopping_items() {
        let user_id = test_user_id();
        let user_id_clone = user_id.clone();
        let mut mock_repo = MockShoppingItemRepo::new();
        mock_repo.expect_get_all().returning(move |_| {
            Ok(vec![
                ShoppingItem::from_repository(
                    Uuid::new_v4(),
                    user_id_clone.clone(),
                    "Milk".to_string(),
                    None,
                    false,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                ),
                ShoppingItem::from_repository(
                    Uuid::new_v4(),
                    user_id_clone.clone(),
                    "Bread".to_string(),
                    None,
                    true,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                ),
            ])
        });

        let use_case = GetAllShoppingItemsUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(GetAllShoppingItemsParams { user_id })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn should_return_empty_when_no_items() {
        let mut mock_repo = MockShoppingItemRepo::new();
        mock_repo.expect_get_all().returning(|_| Ok(vec![]));

        let use_case = GetAllShoppingItemsUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(GetAllShoppingItemsParams {
                user_id: test_user_id(),
            })
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
