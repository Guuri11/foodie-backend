use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::errors::RepositoryError;
use crate::domain::logger::Logger;
use crate::domain::shopping_item::errors::ShoppingItemError;
use crate::domain::shopping_item::repository::ShoppingItemRepository;
use crate::domain::shopping_item::use_cases::delete::{
    DeleteShoppingItemParams, DeleteShoppingItemUseCase,
};

pub struct DeleteShoppingItemUseCaseImpl {
    pub repository: Arc<dyn ShoppingItemRepository>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl DeleteShoppingItemUseCase for DeleteShoppingItemUseCaseImpl {
    async fn execute(&self, params: DeleteShoppingItemParams) -> Result<(), ShoppingItemError> {
        self.logger
            .info(&format!("Deleting shopping item: {}", params.id));

        // Verify it exists
        self.repository
            .get_by_id(params.id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => ShoppingItemError::NotFound,
                other => ShoppingItemError::Repository(other),
            })?;

        self.repository.delete(params.id).await?;

        self.logger
            .info(&format!("Shopping item deleted: {}", params.id));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shopping_item::model::ShoppingItem;
    use mockall::mock;
    use uuid::Uuid;

    mock! {
        pub ShoppingItemRepo {}

        #[async_trait]
        impl ShoppingItemRepository for ShoppingItemRepo {
            async fn get_all(&self) -> Result<Vec<ShoppingItem>, RepositoryError>;
            async fn get_by_id(&self, id: Uuid) -> Result<ShoppingItem, RepositoryError>;
            async fn find_by_product_id(&self, product_id: Uuid) -> Result<Option<ShoppingItem>, RepositoryError>;
            async fn save(&self, item: &ShoppingItem) -> Result<(), RepositoryError>;
            async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
            async fn delete_by_product_id(&self, product_id: Uuid) -> Result<(), RepositoryError>;
            async fn delete_bought(&self) -> Result<u64, RepositoryError>;
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
    async fn should_delete_existing_shopping_item() {
        let item_id = Uuid::new_v4();
        let mut mock_repo = MockShoppingItemRepo::new();

        mock_repo.expect_get_by_id().returning(move |_| {
            Ok(ShoppingItem::from_repository(
                item_id,
                "Milk".to_string(),
                None,
                false,
                chrono::Utc::now(),
                chrono::Utc::now(),
            ))
        });
        mock_repo.expect_delete().returning(|_| Ok(()));

        let use_case = DeleteShoppingItemUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(DeleteShoppingItemParams { id: item_id })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_not_found_when_deleting_nonexistent() {
        let mut mock_repo = MockShoppingItemRepo::new();
        mock_repo
            .expect_get_by_id()
            .returning(|_| Err(RepositoryError::NotFound));

        let use_case = DeleteShoppingItemUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(DeleteShoppingItemParams { id: Uuid::new_v4() })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ShoppingItemError::NotFound));
    }
}
