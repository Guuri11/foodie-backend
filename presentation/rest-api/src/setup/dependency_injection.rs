use std::sync::Arc;

use logger::TracingLogger;
use persistence::product::repository::ProductRepositoryPostgres;

use openai::client::OpenAIClient;
use openai::expiry_estimator::ExpiryEstimatorOpenAI;
use openai::product_identifier::ProductIdentifierOpenAI;
use openai::receipt_scanner::ReceiptScannerOpenAI;
use openai::suggestion_generator::SuggestionGeneratorOpenAI;

use business::application::product::create::CreateProductUseCaseImpl;
use business::application::product::delete::DeleteProductUseCaseImpl;
use business::application::product::estimate_expiry::EstimateExpiryUseCaseImpl;
use business::application::product::get_all::GetAllProductsUseCaseImpl;
use business::application::product::get_by_id::GetProductByIdUseCaseImpl;
use business::application::product::identify::IdentifyProductUseCaseImpl;
use business::application::product::scan_receipt::ScanReceiptUseCaseImpl;
use business::application::product::update::UpdateProductUseCaseImpl;
use business::application::suggestion::generate::GenerateSuggestionsUseCaseImpl;

use crate::config::openai_config::OpenAIConfig;

pub struct DependencyContainer {
    pub health_api: crate::api::health::routes::Api,
    pub product_api: crate::api::product::routes::ProductApi,
    pub suggestion_api: crate::api::suggestion::routes::SuggestionApi,
}

impl DependencyContainer {
    pub async fn new(pool: sqlx::PgPool) -> anyhow::Result<Self> {
        let logger = Arc::new(TracingLogger);
        let health_api = crate::api::health::routes::Api::new();

        // Infrastructure adapters
        let product_repository = Arc::new(ProductRepositoryPostgres::new(pool));

        let openai_config = OpenAIConfig::from_env();
        let openai_client = OpenAIClient::new(openai_config.api_key.clone());
        let openai_client_2 = OpenAIClient::new(openai_config.api_key.clone());
        let openai_client_3 = OpenAIClient::new(openai_config.api_key.clone());
        let openai_client_4 = OpenAIClient::new(openai_config.api_key);

        let expiry_estimator = Arc::new(ExpiryEstimatorOpenAI::new(openai_client));
        let product_identifier = Arc::new(ProductIdentifierOpenAI::new(openai_client_2));
        let receipt_scanner = Arc::new(ReceiptScannerOpenAI::new(openai_client_3));
        let suggestion_generator = Arc::new(SuggestionGeneratorOpenAI::new(openai_client_4));

        // Product use cases
        let create_use_case = Arc::new(CreateProductUseCaseImpl {
            repository: product_repository.clone(),
            logger: logger.clone(),
        });
        let get_all_use_case = Arc::new(GetAllProductsUseCaseImpl {
            repository: product_repository.clone(),
            logger: logger.clone(),
        });
        let get_by_id_use_case = Arc::new(GetProductByIdUseCaseImpl {
            repository: product_repository.clone(),
            logger: logger.clone(),
        });
        let update_use_case = Arc::new(UpdateProductUseCaseImpl {
            repository: product_repository.clone(),
            logger: logger.clone(),
        });
        let delete_use_case = Arc::new(DeleteProductUseCaseImpl {
            repository: product_repository.clone(),
            logger: logger.clone(),
        });
        let estimate_expiry_use_case = Arc::new(EstimateExpiryUseCaseImpl {
            repository: product_repository.clone(),
            estimator: expiry_estimator.clone(),
            logger: logger.clone(),
        });
        let identify_use_case = Arc::new(IdentifyProductUseCaseImpl {
            identifier: product_identifier,
            logger: logger.clone(),
        });
        let scan_receipt_use_case = Arc::new(ScanReceiptUseCaseImpl {
            scanner: receipt_scanner,
            logger: logger.clone(),
        });

        // Suggestion use cases
        let generate_suggestions_use_case = Arc::new(GenerateSuggestionsUseCaseImpl {
            repository: product_repository,
            generator: suggestion_generator,
            logger,
        });

        let product_api = crate::api::product::routes::ProductApi::new(
            create_use_case,
            get_all_use_case,
            get_by_id_use_case,
            update_use_case,
            delete_use_case,
            estimate_expiry_use_case,
            expiry_estimator,
            identify_use_case,
            scan_receipt_use_case,
        );

        let suggestion_api =
            crate::api::suggestion::routes::SuggestionApi::new(generate_suggestions_use_case);

        Ok(Self {
            health_api,
            product_api,
            suggestion_api,
        })
    }
}
