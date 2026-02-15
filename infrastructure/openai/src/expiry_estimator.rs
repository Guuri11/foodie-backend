use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde_json::json;

use business::domain::product::services::{Confidence, ExpiryEstimation, ExpiryEstimatorService};

use crate::client::OpenAIClient;

const SYSTEM_PROMPT: &str = r#"You are an expiry date estimator for a Spanish kitchen inventory app.
Given a product name, its current status, and storage location, estimate how long until it expires.

Rules:
1. Return ONLY a JSON object with these fields:
   - "daysUntilExpiry": number of days from TODAY until the product expires (integer)
   - "confidence": "high" (well-known products), "medium" (reasonable guess), "low" (uncertain), or "none" (cannot estimate)

2. Consider the product's current status:
   - "new": Unopened, sealed package
   - "opened": Package has been opened
   - "almost_empty": Nearly finished
   - "finished": Empty (should not be called, but treat as 0 days)

3. Consider storage location (affects shelf life):
   - "fridge": Refrigerated (extends perishables)
   - "freezer": Frozen (significantly extends shelf life)
   - "pantry": Room temperature (dry goods)
   - undefined: Assume room temperature

4. Base estimates on food safety guidelines, not "best before" dates.

5. If you cannot estimate (e.g., too generic like "food"), return:
   {"daysUntilExpiry":null,"confidence":"none"}

Examples:
{"daysUntilExpiry":3,"confidence":"high"}  // Opened milk in fridge
{"daysUntilExpiry":180,"confidence":"high"} // New rice in pantry
{"daysUntilExpiry":2,"confidence":"high"}  // Opened chicken in fridge
{"daysUntilExpiry":null,"confidence":"none"} // Cannot estimate"#;

pub struct ExpiryEstimatorOpenAI {
    client: OpenAIClient,
    cache: Mutex<HashMap<String, ExpiryEstimation>>,
}

impl ExpiryEstimatorOpenAI {
    pub fn new(client: OpenAIClient) -> Self {
        Self {
            client,
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn build_cache_key(product_name: &str, status: &str, location: Option<&str>) -> String {
        format!(
            "{}|{}|{}",
            product_name.to_lowercase(),
            status,
            location.unwrap_or("none")
        )
    }

    fn build_user_prompt(product_name: &str, status: &str, location: Option<&str>) -> String {
        let mut parts = vec![
            format!("Product: {}", product_name),
            format!("Status: {}", status),
        ];
        if let Some(loc) = location {
            parts.push(format!("Location: {}", loc));
        }
        parts.push("Estimate expiry date.".to_string());
        parts.join("\n")
    }

    fn parse_response(content: &str) -> ExpiryEstimation {
        let json_match = regex::Regex::new(r"\{[\s\S]*\}")
            .ok()
            .and_then(|re| re.find(content));

        let json_str = match json_match {
            Some(m) => m.as_str(),
            None => {
                return ExpiryEstimation {
                    date: None,
                    confidence: Confidence::None,
                };
            }
        };

        let parsed: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => {
                return ExpiryEstimation {
                    date: None,
                    confidence: Confidence::None,
                };
            }
        };

        let confidence = match parsed.get("confidence").and_then(|c| c.as_str()) {
            Some("high") => Confidence::High,
            Some("medium") => Confidence::Medium,
            Some("low") => Confidence::Low,
            _ => Confidence::None,
        };

        let date = parsed
            .get("daysUntilExpiry")
            .and_then(|d| d.as_i64())
            .map(|days| Utc::now() + Duration::days(days));

        ExpiryEstimation { date, confidence }
    }
}

#[async_trait]
impl ExpiryEstimatorService for ExpiryEstimatorOpenAI {
    async fn estimate_expiry_date(
        &self,
        product_name: &str,
        status: &str,
        location: Option<String>,
    ) -> ExpiryEstimation {
        let cache_key = Self::build_cache_key(product_name, status, location.as_deref());

        // Check cache
        if let Ok(cache) = self.cache.lock()
            && let Some(cached) = cache.get(&cache_key)
        {
            return cached.clone();
        }

        let user_prompt = Self::build_user_prompt(product_name, status, location.as_deref());

        let body = json!({
            "model": "gpt-4o",
            "input": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": user_prompt},
            ],
            "temperature": 0.1,
        });

        let response = self
            .client
            .client
            .post(self.client.responses_url())
            .header("Content-Type", "application/json")
            .header("Authorization", self.client.auth_header())
            .json(&body)
            .send()
            .await;

        let estimation = match response {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let text = data["output"]
                            .as_array()
                            .and_then(|outputs| outputs.iter().find(|o| o["type"] == "message"))
                            .and_then(|msg| msg["content"].as_array())
                            .and_then(|contents| {
                                contents.iter().find(|c| c["type"] == "output_text")
                            })
                            .and_then(|c| c["text"].as_str());

                        match text {
                            Some(t) => Self::parse_response(t),
                            None => ExpiryEstimation {
                                date: None,
                                confidence: Confidence::None,
                            },
                        }
                    }
                    Err(_) => ExpiryEstimation {
                        date: None,
                        confidence: Confidence::None,
                    },
                }
            }
            _ => ExpiryEstimation {
                date: None,
                confidence: Confidence::None,
            },
        };

        // Cache result
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cache_key, estimation.clone());
        }

        estimation
    }
}
