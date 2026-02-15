/// Configuration for OpenAI API access.
pub struct OpenAIConfig {
    pub api_key: String,
}

impl OpenAIConfig {
    pub fn from_env() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY environment variable must be set");
        Self { api_key }
    }
}
