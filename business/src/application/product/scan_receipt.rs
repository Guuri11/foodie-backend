use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::logger::Logger;
use crate::domain::product::errors::ProductError;
use crate::domain::product::services::{ReceiptScanResult, ReceiptScannerService};
use crate::domain::product::use_cases::scan_receipt::{ScanReceiptParams, ScanReceiptUseCase};

pub struct ScanReceiptUseCaseImpl {
    pub scanner: Arc<dyn ReceiptScannerService>,
    pub logger: Arc<dyn Logger>,
}

#[async_trait]
impl ScanReceiptUseCase for ScanReceiptUseCaseImpl {
    async fn execute(&self, params: ScanReceiptParams) -> Result<ReceiptScanResult, ProductError> {
        self.logger.info("Scanning receipt image");

        let result = self.scanner.scan(&params.image_base64).await?;

        self.logger.info(&format!(
            "Receipt scanned: {} items found",
            result.items.len()
        ));

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::product::services::{IdentificationConfidence, ReceiptItem};
    use mockall::mock;

    mock! {
        pub ReceiptScanner {}

        #[async_trait]
        impl ReceiptScannerService for ReceiptScanner {
            async fn scan(&self, image_base64: &str) -> Result<ReceiptScanResult, ProductError>;
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
    async fn should_return_items_when_receipt_scanned_successfully() {
        let mut mock_scanner = MockReceiptScanner::new();
        mock_scanner.expect_scan().returning(|_| {
            Ok(ReceiptScanResult {
                items: vec![
                    ReceiptItem {
                        name: "Leche entera".to_string(),
                        confidence: IdentificationConfidence::High,
                    },
                    ReceiptItem {
                        name: "Pan de molde".to_string(),
                        confidence: IdentificationConfidence::High,
                    },
                    ReceiptItem {
                        name: "Manzanas".to_string(),
                        confidence: IdentificationConfidence::Low,
                    },
                ],
            })
        });

        let use_case = ScanReceiptUseCaseImpl {
            scanner: Arc::new(mock_scanner),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(ScanReceiptParams {
                image_base64: "receipt_image_data".to_string(),
            })
            .await;

        assert!(result.is_ok());
        let scan_result = result.unwrap();
        assert_eq!(scan_result.items.len(), 3);
        assert_eq!(scan_result.items[0].name, "Leche entera");
    }

    #[tokio::test]
    async fn should_return_empty_items_when_receipt_has_no_products() {
        let mut mock_scanner = MockReceiptScanner::new();
        mock_scanner
            .expect_scan()
            .returning(|_| Ok(ReceiptScanResult { items: vec![] }));

        let use_case = ScanReceiptUseCaseImpl {
            scanner: Arc::new(mock_scanner),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(ScanReceiptParams {
                image_base64: "blank_receipt".to_string(),
            })
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().items.is_empty());
    }

    #[tokio::test]
    async fn should_return_error_when_scan_fails() {
        let mut mock_scanner = MockReceiptScanner::new();
        mock_scanner
            .expect_scan()
            .returning(|_| Err(ProductError::ScanFailed));

        let use_case = ScanReceiptUseCaseImpl {
            scanner: Arc::new(mock_scanner),
            logger: mock_logger(),
        };

        let result = use_case
            .execute(ScanReceiptParams {
                image_base64: "corrupted_image".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProductError::ScanFailed));
    }
}
