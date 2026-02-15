use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::errors::RepositoryError;
use crate::domain::logger::Logger;
use crate::domain::shopping_item::errors::ShoppingItemError;
use crate::domain::shopping_item::model::ShoppingItem;
use crate::domain::shopping_item::repository::ShoppingItemRepository;
use crate::domain::shopping_item::use_cases::update::{
    UpdateShoppingItemParams, UpdateShoppingItemUseCase,
};

pub struct UpdateShoppingItemUseCaseImpl {
    pub repository: Arc<dyn ShoppingItemRepository>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl UpdateShoppingItemUseCase for UpdateShoppingItemUseCaseImpl {
    async fn execute(
        &self,
        params: UpdateShoppingItemParams,
    ) -> Result<ShoppingItem, ShoppingItemError> {
        self.logger
            .info(&format!("Updating shopping item: {}", params.id));

        let existing = self
            .repository
            .get_by_id(params.id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => ShoppingItemError::NotFound,
                other => ShoppingItemError::Repository(other),
            })?;

        let name = match params.name {
            Some(ref n) if n.trim().is_empty() => return Err(ShoppingItemError::NameEmpty),
            Some(n) => n,
            None => existing.name,
        };

        let is_bought = params.is_bought.unwrap_or(existing.is_bought);

        let updated = ShoppingItem::from_repository(
            existing.id,
            name,
            existing.product_id,
            is_bought,
            existing.created_at,
            chrono::Utc::now(),
        );

        self.repository.save(&updated).await?;

        self.logger
            .info(&format!("Shopping item updated: {}", updated.id));
        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    async fn should_toggle_bought_status() {
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
        mock_repo.expect_save().returning(|_| Ok(()));

        let use_case = UpdateShoppingItemUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateShoppingItemParams {
                id: item_id,
                name: None,
                is_bought: Some(true),
            })
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_bought);
    }

    #[tokio::test]
    async fn should_update_name() {
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
        mock_repo.expect_save().returning(|_| Ok(()));

        let use_case = UpdateShoppingItemUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateShoppingItemParams {
                id: item_id,
                name: Some("Whole Milk".to_string()),
                is_bought: None,
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Whole Milk");
    }

    #[tokio::test]
    async fn should_return_not_found_when_item_does_not_exist() {
        let mut mock_repo = MockShoppingItemRepo::new();
        mock_repo
            .expect_get_by_id()
            .returning(|_| Err(RepositoryError::NotFound));

        let use_case = UpdateShoppingItemUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateShoppingItemParams {
                id: Uuid::new_v4(),
                name: None,
                is_bought: Some(true),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ShoppingItemError::NotFound));
    }

    #[tokio::test]
    async fn should_reject_update_when_name_empty() {
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

        let use_case = UpdateShoppingItemUseCaseImpl {
            repository: Arc::new(mock_repo),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(UpdateShoppingItemParams {
                id: item_id,
                name: Some("".to_string()),
                is_bought: None,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ShoppingItemError::NameEmpty));
    }
}
