use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::logger::Logger;
use crate::domain::shopping_item::errors::ShoppingItemError;
use crate::domain::shopping_item::repository::ShoppingItemRepository;
use crate::domain::shopping_item::use_cases::clear_bought::{
    ClearBoughtItemsParams, ClearBoughtItemsUseCase,
};

pub struct ClearBoughtItemsUseCaseImpl {
    pub repository: Arc<dyn ShoppingItemRepository>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl ClearBoughtItemsUseCase for ClearBoughtItemsUseCaseImpl {
    async fn execute(&self, params: ClearBoughtItemsParams) -> Result<u64, ShoppingItemError> {
        self.logger.info("Clearing bought shopping items");

        let count = self.repository.delete_bought(&params.user_id).await?;

        self.logger
            .info(&format!("Cleared {} bought shopping items", count));
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::RepositoryError;
    use crate::domain::shared::value_objects::UserId;
    use crate::domain::shopping_item::model::ShoppingItem;
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
    async fn should_delete_only_bought_items() {
        let mut mock_repo = MockShoppingItemRepo::new();
        mock_repo.expect_delete_bought().returning(|_| Ok(3));

        let use_case = ClearBoughtItemsUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(ClearBoughtItemsParams {
                user_id: test_user_id(),
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[tokio::test]
    async fn should_return_zero_when_no_bought_items() {
        let mut mock_repo = MockShoppingItemRepo::new();
        mock_repo.expect_delete_bought().returning(|_| Ok(0));

        let use_case = ClearBoughtItemsUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(ClearBoughtItemsParams {
                user_id: test_user_id(),
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
