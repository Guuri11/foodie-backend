use reqwest::Client;

/// Shared OpenAI HTTP client configuration.
pub struct OpenAIClient {
    pub client: Client,
    pub api_key: String,
    pub base_url: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    /// Builds the authorization header value.
    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    /// Returns the chat completions endpoint URL.
    pub fn chat_completions_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    /// Returns the responses endpoint URL.
    pub fn responses_url(&self) -> String {
        format!("{}/responses", self.base_url)
    }
}
