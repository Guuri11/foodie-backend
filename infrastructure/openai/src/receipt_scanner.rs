use async_trait::async_trait;
use serde_json::json;

use business::domain::product::errors::ProductError;
use business::domain::product::services::{
    IdentificationConfidence, ReceiptItem, ReceiptScanResult, ReceiptScannerService,
};

use crate::client::OpenAIClient;

const SYSTEM_PROMPT: &str = r#"You are a receipt scanner for a Spanish kitchen inventory app.
Extract product names from this supermarket receipt image.
Return ONLY a JSON array of objects with "name" and "confidence" fields.
- "name": the product name in Spanish, cleaned up (no brand, no weight, no price)
- "confidence": "high" if clearly readable, "low" if uncertain
- Filter out non-food items (bags, discounts, totals, store info)
- Keep it simple: "Leche entera", not "LECHE ENTERA HACENDADO 1L 0.89"

Example output:
[{"name":"Leche entera","confidence":"high"},{"name":"Pan de molde","confidence":"high"},{"name":"Manzanas","confidence":"low"}]"#;

pub struct ReceiptScannerOpenAI {
    client: OpenAIClient,
}

impl ReceiptScannerOpenAI {
    pub fn new(client: OpenAIClient) -> Self {
        Self { client }
    }

    fn to_clean_data_url(raw: &str) -> String {
        let stripped = regex::Regex::new(r"^data:image/[a-z]+;base64,")
            .map(|re| re.replace(raw, "").to_string())
            .unwrap_or_else(|_| raw.to_string());
        let clean: String = stripped.chars().filter(|c| !c.is_whitespace()).collect();
        format!("data:image/jpeg;base64,{}", clean)
    }

    fn parse_response(content: &str) -> Result<ReceiptScanResult, ProductError> {
        let json_match = regex::Regex::new(r"\[[\s\S]*\]")
            .ok()
            .and_then(|re| re.find(content));

        let json_str = json_match
            .map(|m| m.as_str())
            .ok_or(ProductError::ScanFailed)?;

        let parsed: Vec<serde_json::Value> =
            serde_json::from_str(json_str).map_err(|_| ProductError::ScanFailed)?;

        let items: Vec<ReceiptItem> = parsed
            .iter()
            .filter_map(|item| {
                let name = item.get("name")?.as_str()?.to_string();
                let confidence = match item.get("confidence").and_then(|c| c.as_str()) {
                    Some("low") => IdentificationConfidence::Low,
                    _ => IdentificationConfidence::High,
                };
                Some(ReceiptItem { name, confidence })
            })
            .collect();

        Ok(ReceiptScanResult { items })
    }
}

#[async_trait]
impl ReceiptScannerService for ReceiptScannerOpenAI {
    async fn scan(&self, image_base64: &str) -> Result<ReceiptScanResult, ProductError> {
        let image_url = Self::to_clean_data_url(image_base64);

        let body = json!({
            "model": "gpt-4o",
            "input": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "input_image",
                            "image_url": image_url,
                            "detail": "high",
                        },
                        {
                            "type": "input_text",
                            "text": "Extract the product names from this receipt.",
                        },
                    ],
                },
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
            .await
            .map_err(|_| ProductError::ScanFailed)?;

        if !response.status().is_success() {
            return Err(ProductError::ScanFailed);
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|_| ProductError::ScanFailed)?;

        let text = data["output"]
            .as_array()
            .and_then(|outputs| outputs.iter().find(|o| o["type"] == "message"))
            .and_then(|msg| msg["content"].as_array())
            .and_then(|contents| contents.iter().find(|c| c["type"] == "output_text"))
            .and_then(|c| c["text"].as_str())
            .ok_or(ProductError::ScanFailed)?;

        Self::parse_response(text)
    }
}
