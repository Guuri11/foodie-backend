use poem::{EndpointExt, Route, Server as PoemServer, listener::TcpListener, middleware::Tracing};
use poem_openapi::OpenApiService;

use crate::{config::app_config::AppConfig, setup::dependency_injection::DependencyContainer};

pub struct Server;

impl Server {
    pub async fn run(config: AppConfig, container: DependencyContainer) -> anyhow::Result<()> {
        let addr = config.server.bind_address();
        let api_service = OpenApiService::new(
            (
                container.health_api,
                container.product_api,
                container.shopping_item_api,
                container.suggestion_api,
            ),
            "Foodie Backend API",
            "0.1.0",
        )
        .server(format!("http://{}", addr));
        let ui = api_service.swagger_ui();
        let spec = api_service.spec_endpoint();
        let app = Route::new()
            .nest("/", api_service)
            .nest("/docs", ui)
            .nest("/openapi.json", spec)
            .with(config.cors)
            .with(Tracing);
        println!("Server running at http://{}", addr);
        println!("Swagger UI at http://{}/docs", addr);
        println!("OpenAPI JSON at http://{}/openapi.json", addr);
        PoemServer::new(TcpListener::bind(&addr)).run(app).await?;
        Ok(())
    }
}
