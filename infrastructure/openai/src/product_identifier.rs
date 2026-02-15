use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use business::domain::product::errors::ProductError;
use business::domain::product::services::{
    IdentificationConfidence, IdentificationMethod, ProductIdentification, ProductIdentifierService,
};
use business::domain::product::value_objects::ProductLocation;

use crate::client::OpenAIClient;

const SYSTEM_PROMPT: &str = r#"You are a product identifier for a Spanish kitchen inventory app.
Identify this single food product from the image.
Return ONLY a JSON object with these fields:
- "name": the product name in Spanish, cleaned up (no brand, no weight, no price)
- "confidence": "high" if clearly identifiable, "low" if uncertain
- "suggestedLocation": where this product is typically stored: "fridge", "pantry", or "freezer" (optional)
- "suggestedQuantity": the quantity if visible on the package, e.g. "1 L", "500 g" (optional)
- If you cannot identify the product at all, return {"name":"","confidence":"low"}

Example outputs:
{"name":"Yogur natural","confidence":"high","suggestedLocation":"fridge","suggestedQuantity":"4 x 125 g"}
{"name":"Arroz","confidence":"high","suggestedLocation":"pantry"}"#;

#[derive(Deserialize)]
struct OpenFoodFactsResponse {
    status: i32,
    product: Option<OpenFoodFactsProduct>,
}

#[derive(Deserialize)]
struct OpenFoodFactsProduct {
    product_name_es: Option<String>,
    product_name: Option<String>,
    quantity: Option<String>,
    categories_tags: Option<Vec<String>>,
}

pub struct ProductIdentifierOpenAI {
    client: OpenAIClient,
}

impl ProductIdentifierOpenAI {
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

    fn parse_image_response(content: &str) -> Result<ProductIdentification, ProductError> {
        let json_match = regex::Regex::new(r"\{[\s\S]*\}")
            .ok()
            .and_then(|re| re.find(content));

        let json_str = json_match
            .map(|m| m.as_str())
            .ok_or(ProductError::IdentificationFailed)?;

        let parsed: serde_json::Value =
            serde_json::from_str(json_str).map_err(|_| ProductError::IdentificationFailed)?;

        let name = parsed
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();

        let confidence = match parsed.get("confidence").and_then(|c| c.as_str()) {
            Some("high") => IdentificationConfidence::High,
            _ => IdentificationConfidence::Low,
        };

        let suggested_location = parsed
            .get("suggestedLocation")
            .and_then(|l| l.as_str())
            .and_then(|l| l.parse::<ProductLocation>().ok());

        let suggested_quantity = parsed
            .get("suggestedQuantity")
            .and_then(|q| q.as_str())
            .map(|q| q.to_string());

        Ok(ProductIdentification {
            name,
            confidence,
            method: IdentificationMethod::Visual,
            suggested_location,
            suggested_quantity,
        })
    }

    fn infer_location_from_categories(categories: &[String]) -> Option<ProductLocation> {
        let joined = categories.join(",").to_lowercase();

        if joined.contains("frozen") || joined.contains("congel") {
            return Some(ProductLocation::Freezer);
        }
        if joined.contains("dairy")
            || joined.contains("lact")
            || joined.contains("fresh")
            || joined.contains("fresc")
            || joined.contains("meat")
            || joined.contains("carn")
            || joined.contains("fish")
            || joined.contains("pescad")
        {
            return Some(ProductLocation::Fridge);
        }

        Some(ProductLocation::Pantry)
    }
}

#[async_trait]
impl ProductIdentifierService for ProductIdentifierOpenAI {
    async fn identify_by_image(
        &self,
        image_base64: &str,
    ) -> Result<ProductIdentification, ProductError> {
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
                            "detail": "low",
                        },
                        {
                            "type": "input_text",
                            "text": "Identify this food product.",
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
            .map_err(|_| ProductError::IdentificationFailed)?;

        if !response.status().is_success() {
            return Err(ProductError::IdentificationFailed);
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|_| ProductError::IdentificationFailed)?;

        let text = data["output"]
            .as_array()
            .and_then(|outputs| outputs.iter().find(|o| o["type"] == "message"))
            .and_then(|msg| msg["content"].as_array())
            .and_then(|contents| contents.iter().find(|c| c["type"] == "output_text"))
            .and_then(|c| c["text"].as_str())
            .ok_or(ProductError::IdentificationFailed)?;

        Self::parse_image_response(text)
    }

    async fn identify_by_barcode(
        &self,
        barcode: &str,
    ) -> Result<ProductIdentification, ProductError> {
        let url = format!(
            "https://world.openfoodfacts.org/api/v2/product/{}.json",
            barcode
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|_| ProductError::IdentificationFailed)?;

        if !response.status().is_success() {
            return Err(ProductError::IdentificationFailed);
        }

        let data: OpenFoodFactsResponse = response
            .json()
            .await
            .map_err(|_| ProductError::IdentificationFailed)?;

        if data.status != 1 {
            return Err(ProductError::IdentificationFailed);
        }

        let product = data.product.ok_or(ProductError::IdentificationFailed)?;

        let name = product
            .product_name_es
            .or(product.product_name)
            .filter(|n| !n.is_empty())
            .ok_or(ProductError::IdentificationFailed)?;

        let suggested_quantity = product.quantity;
        let categories = product.categories_tags.unwrap_or_default();
        let suggested_location = Self::infer_location_from_categories(&categories);

        Ok(ProductIdentification {
            name,
            confidence: IdentificationConfidence::High,
            method: IdentificationMethod::Barcode,
            suggested_location,
            suggested_quantity,
        })
    }
}
