use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::logger::Logger;
use crate::domain::product::errors::ProductError;
use crate::domain::product::services::{ProductIdentification, ProductIdentifierService};
use crate::domain::product::use_cases::identify::{
    IdentifyByBarcodeParams, IdentifyByImageParams, IdentifyProductUseCase,
};

pub struct IdentifyProductUseCaseImpl {
    pub identifier: Arc<dyn ProductIdentifierService>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl IdentifyProductUseCase for IdentifyProductUseCaseImpl {
    async fn execute_by_image(
        &self,
        params: IdentifyByImageParams,
    ) -> Result<ProductIdentification, ProductError> {
        self.logger.info("Identifying product by image");

        let result = self
            .identifier
            .identify_by_image(&params.image_base64)
            .await?;

        self.logger.info(&format!(
            "Product identified by image: {} (confidence: {})",
            result.name, result.confidence
        ));

        Ok(result)
    }

    async fn execute_by_barcode(
        &self,
        params: IdentifyByBarcodeParams,
    ) -> Result<ProductIdentification, ProductError> {
        self.logger.info(&format!(
            "Identifying product by barcode: {}",
            params.barcode
        ));

        let result = self.identifier.identify_by_barcode(&params.barcode).await?;

        self.logger.info(&format!(
            "Product identified by barcode: {} (confidence: {})",
            result.name, result.confidence
        ));

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::product::services::{
        IdentificationConfidence, IdentificationMethod, ProductIdentification,
    };
    use crate::domain::product::value_objects::ProductLocation;
    use mockall::mock;

    mock! {
        pub ProductIdentifier {}

        #[async_trait]
        impl ProductIdentifierService for ProductIdentifier {
            async fn identify_by_image(
                &self,
                image_base64: &str,
            ) -> Result<ProductIdentification, ProductError>;

            async fn identify_by_barcode(
                &self,
                barcode: &str,
            ) -> Result<ProductIdentification, ProductError>;
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
    async fn should_identify_product_when_image_is_clear() {
        let mut mock_identifier = MockProductIdentifier::new();
        mock_identifier.expect_identify_by_image().returning(|_| {
            Ok(ProductIdentification {
                name: "Yogur natural".to_string(),
                confidence: IdentificationConfidence::High,
                method: IdentificationMethod::Visual,
                suggested_location: Some(ProductLocation::Fridge),
                suggested_quantity: Some("4 x 125 g".to_string()),
            })
        });

        let use_case = IdentifyProductUseCaseImpl {
            identifier: Arc::new(mock_identifier),
            logger: mock_logger(),
        };

        let result = use_case
            .execute_by_image(IdentifyByImageParams {
                image_base64: "base64data".to_string(),
            })
            .await;

        assert!(result.is_ok());
        let identification = result.unwrap();
        assert_eq!(identification.name, "Yogur natural");
        assert_eq!(identification.confidence, IdentificationConfidence::High);
        assert_eq!(identification.method, IdentificationMethod::Visual);
    }

    #[tokio::test]
    async fn should_identify_product_when_barcode_found() {
        let mut mock_identifier = MockProductIdentifier::new();
        mock_identifier.expect_identify_by_barcode().returning(|_| {
            Ok(ProductIdentification {
                name: "Leche entera".to_string(),
                confidence: IdentificationConfidence::High,
                method: IdentificationMethod::Barcode,
                suggested_location: Some(ProductLocation::Fridge),
                suggested_quantity: Some("1 L".to_string()),
            })
        });

        let use_case = IdentifyProductUseCaseImpl {
            identifier: Arc::new(mock_identifier),
            logger: mock_logger(),
        };

        let result = use_case
            .execute_by_barcode(IdentifyByBarcodeParams {
                barcode: "8410000810004".to_string(),
            })
            .await;

        assert!(result.is_ok());
        let identification = result.unwrap();
        assert_eq!(identification.name, "Leche entera");
        assert_eq!(identification.method, IdentificationMethod::Barcode);
    }

    #[tokio::test]
    async fn should_return_error_when_image_identification_fails() {
        let mut mock_identifier = MockProductIdentifier::new();
        mock_identifier
            .expect_identify_by_image()
            .returning(|_| Err(ProductError::IdentificationFailed));

        let use_case = IdentifyProductUseCaseImpl {
            identifier: Arc::new(mock_identifier),
            logger: mock_logger(),
        };

        let result = use_case
            .execute_by_image(IdentifyByImageParams {
                image_base64: "bad_data".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProductError::IdentificationFailed
        ));
    }

    #[tokio::test]
    async fn should_return_error_when_barcode_not_found() {
        let mut mock_identifier = MockProductIdentifier::new();
        mock_identifier
            .expect_identify_by_barcode()
            .returning(|_| Err(ProductError::IdentificationFailed));

        let use_case = IdentifyProductUseCaseImpl {
            identifier: Arc::new(mock_identifier),
            logger: mock_logger(),
        };

        let result = use_case
            .execute_by_barcode(IdentifyByBarcodeParams {
                barcode: "0000000000000".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProductError::IdentificationFailed
        ));
    }
}
