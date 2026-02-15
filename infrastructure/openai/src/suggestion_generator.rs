use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;

use business::domain::product::model::Product;
use business::domain::product::urgency::{days_until_expiry, get_urgency_level};
use business::domain::suggestion::errors::SuggestionError;
use business::domain::suggestion::model::{Suggestion, SuggestionIngredient, TimeRange};
use business::domain::suggestion::services::SuggestionGeneratorService;

use crate::client::OpenAIClient;

const SYSTEM_PROMPT: &str = r#"You are a helpful cooking assistant for a Spanish kitchen app called Foodie.
Your goal: help tired users decide what to cook quickly, prioritizing ingredients that are expiring soon.

Core principles:
- Keep suggestions SIMPLE (max 30 min cooking time)
- Prioritize products expiring soon
- Use realistic ingredient combinations
- Be calm and clear - this is for people who are tired
- Suggest 3-5 recipes maximum
- Focus on common Spanish/Mediterranean dishes when possible

Return ONLY valid JSON array, no additional text."#;

pub struct SuggestionGeneratorOpenAI {
    client: OpenAIClient,
}

impl SuggestionGeneratorOpenAI {
    pub fn new(client: OpenAIClient) -> Self {
        Self { client }
    }

    fn build_prompt(products: &[Product], limit: usize) -> String {
        let product_list: String = products
            .iter()
            .map(|p| {
                let urgency = get_urgency_level(p);
                let days = days_until_expiry(p);
                let days_text = match days {
                    Some(d) => format!("expires in {} days", d),
                    None => "no expiry date".to_string(),
                };
                format!("- {} [id:{}] ({}, {})", p.name, p.id, urgency, days_text)
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Given these products from the user's pantry, suggest {} simple recipes they can make TODAY.

PRODUCTS (sorted by urgency):
{}

Requirements:
- Return {} suggestions maximum
- Prioritize recipes using products expiring soon (use_today, use_soon)
- Keep recipes SIMPLE and realistic
- Estimate time: "quick" (~10min), "medium" (~20min), "long" (~30min)
- Provide 3-4 brief steps per recipe
- Use products from the list above

Return JSON array with this EXACT structure:
[
  {{
    "title": "Recipe name in Spanish",
    "description": "Brief description mentioning urgent ingredients if any",
    "estimatedTime": "quick" | "medium" | "long",
    "ingredients": [
      {{
        "productId": "product-id-from-list",
        "productName": "Product name",
        "isUrgent": true | false
      }}
    ],
    "steps": ["Step 1", "Step 2", "Step 3"]
  }}
]"#,
            limit, product_list, limit
        )
    }

    fn parse_response(
        content: &str,
        products: &[Product],
    ) -> Result<Vec<Suggestion>, SuggestionError> {
        // Remove markdown code blocks if present
        let mut json_text = content.trim().to_string();
        if json_text.starts_with("```json") {
            json_text = json_text
                .replace("```json", "")
                .replace("```", "")
                .trim()
                .to_string();
        } else if json_text.starts_with("```") {
            json_text = json_text.replace("```", "").trim().to_string();
        }

        let parsed: Vec<serde_json::Value> =
            serde_json::from_str(&json_text).map_err(|_| SuggestionError::GenerationFailed)?;

        let mut suggestions = Vec::new();

        for (index, item) in parsed.iter().enumerate() {
            let title = item
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();

            let description = item
                .get("description")
                .and_then(|d| d.as_str())
                .map(|d| d.to_string());

            let estimated_time = match item.get("estimatedTime").and_then(|t| t.as_str()) {
                Some("quick") => TimeRange::Quick,
                Some("medium") => TimeRange::Medium,
                Some("long") => TimeRange::Long,
                _ => TimeRange::Medium,
            };

            let ingredients: Vec<SuggestionIngredient> = item
                .get("ingredients")
                .and_then(|i| i.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|ing| {
                            let product_id = ing.get("productId")?.as_str()?.to_string();
                            let product_name = ing.get("productName")?.as_str()?.to_string();
                            let is_urgent = ing
                                .get("isUrgent")
                                .and_then(|u| u.as_bool())
                                .unwrap_or(false);

                            let quantity = products
                                .iter()
                                .find(|p| p.id.to_string() == product_id)
                                .and_then(|p| p.quantity.clone());

                            Some(SuggestionIngredient {
                                product_id,
                                product_name,
                                quantity,
                                is_urgent,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            let steps: Option<Vec<String>> =
                item.get("steps").and_then(|s| s.as_array()).map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect()
                });

            if title.is_empty() || ingredients.is_empty() {
                continue;
            }

            let urgent_ingredients: Vec<String> = ingredients
                .iter()
                .filter(|ing| ing.is_urgent)
                .map(|ing| ing.product_id.clone())
                .collect();

            suggestions.push(Suggestion {
                id: format!("openai-{}-{}", Utc::now().timestamp_millis(), index),
                title,
                description,
                estimated_time,
                ingredients,
                urgent_ingredients,
                steps,
                created_at: Utc::now(),
            });
        }

        Ok(suggestions)
    }
}

#[async_trait]
impl SuggestionGeneratorService for SuggestionGeneratorOpenAI {
    async fn generate(
        &self,
        products: &[Product],
        limit: usize,
    ) -> Result<Vec<Suggestion>, SuggestionError> {
        if products.is_empty() {
            return Ok(vec![]);
        }

        let prompt = Self::build_prompt(products, limit);

        let body = json!({
            "model": "gpt-4o-mini",
            "messages": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": prompt},
            ],
            "temperature": 0.7,
            "max_tokens": 2000,
        });

        let response = self
            .client
            .client
            .post(self.client.chat_completions_url())
            .header("Content-Type", "application/json")
            .header("Authorization", self.client.auth_header())
            .json(&body)
            .send()
            .await
            .map_err(|_| SuggestionError::GenerationFailed)?;

        if !response.status().is_success() {
            return Err(SuggestionError::GenerationFailed);
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|_| SuggestionError::GenerationFailed)?;

        let content = data["choices"]
            .as_array()
            .and_then(|choices| choices.first())
            .and_then(|choice| choice["message"]["content"].as_str())
            .ok_or(SuggestionError::GenerationFailed)?;

        Self::parse_response(content, products)
    }
}
