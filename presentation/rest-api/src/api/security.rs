use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation, decode, decode_header};
use once_cell::sync::Lazy;
use poem::Request;
use poem_openapi::SecurityScheme;
use serde::Deserialize;

use crate::config::firebase_config::FirebaseConfig;

const GOOGLE_CERTS_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";
const CACHE_TTL: Duration = Duration::from_secs(3600);

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FirebaseClaims {
    sub: String,
    email: Option<String>,
    iss: String,
    aud: String,
    exp: u64,
    iat: u64,
}

struct CachedCerts {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
}

static CERTS_CACHE: Lazy<RwLock<Option<CachedCerts>>> = Lazy::new(|| RwLock::new(None));

async fn fetch_google_certs() -> Result<HashMap<String, DecodingKey>, String> {
    let response: HashMap<String, String> = reqwest::get(GOOGLE_CERTS_URL)
        .await
        .map_err(|e| format!("auth.certs_fetch_failed: {e}"))?
        .json()
        .await
        .map_err(|e| format!("auth.certs_parse_failed: {e}"))?;

    let mut keys = HashMap::new();
    for (kid, pem) in response {
        let key = DecodingKey::from_rsa_pem(pem.as_bytes())
            .map_err(|e| format!("auth.cert_decode_failed: {e}"))?;
        keys.insert(kid, key);
    }

    Ok(keys)
}

async fn get_decoding_keys() -> Result<HashMap<String, DecodingKey>, String> {
    // Check cache first
    {
        let cache = CERTS_CACHE
            .read()
            .map_err(|e| format!("auth.cache_read_failed: {e}"))?;
        if let Some(cached) = cache.as_ref()
            && cached.fetched_at.elapsed() < CACHE_TTL
        {
            return Ok(cached.keys.clone());
        }
    }

    // Fetch fresh certs
    let keys = fetch_google_certs().await?;

    // Update cache
    {
        let mut cache = CERTS_CACHE
            .write()
            .map_err(|e| format!("auth.cache_write_failed: {e}"))?;
        *cache = Some(CachedCerts {
            keys: keys.clone(),
            fetched_at: Instant::now(),
        });
    }

    Ok(keys)
}

fn extract_uid_from_token(token: &str) -> Result<String, String> {
    // We need the kid from the header to find the right cert
    let header: Header =
        decode_header(token).map_err(|e| format!("auth.invalid_token_header: {e}"))?;

    let kid = header.kid.ok_or("auth.missing_kid")?;

    // This is called from an async context, but we need sync access to cached keys.
    // The caller must ensure keys are pre-fetched.
    let cache = CERTS_CACHE
        .read()
        .map_err(|e| format!("auth.cache_read_failed: {e}"))?;
    let cached = cache.as_ref().ok_or("auth.certs_not_loaded")?;

    let decoding_key = cached.keys.get(&kid).ok_or("auth.unknown_kid")?;

    let config = FirebaseConfig::from_env();
    let expected_issuer = format!("https://securetoken.google.com/{}", config.project_id);

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&config.project_id]);
    validation.set_issuer(&[&expected_issuer]);
    validation.validate_exp = true;

    let token_data = decode::<FirebaseClaims>(token, decoding_key, &validation)
        .map_err(|e| format!("auth.token_validation_failed: {e}"))?;

    Ok(token_data.claims.sub)
}

/// Firebase Bearer token authentication
#[derive(SecurityScheme)]
#[oai(
    ty = "bearer",
    bearer_format = "JWT",
    checker = "firebase_bearer_checker"
)]
#[allow(dead_code)]
pub struct FirebaseBearer(pub String);

async fn firebase_bearer_checker(
    _req: &Request,
    bearer: poem_openapi::auth::Bearer,
) -> Option<String> {
    // Ensure certs are loaded/refreshed
    if let Err(e) = get_decoding_keys().await {
        tracing::error!("Failed to fetch Google certs: {e}");
        return None;
    }

    match extract_uid_from_token(&bearer.token) {
        Ok(uid) => Some(uid),
        Err(e) => {
            tracing::warn!("Firebase auth failed: {e}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_reject_token_when_header_is_malformed() {
        // Arrange - Ensure cache has some dummy data so we don't fail on missing cache
        let result = extract_uid_from_token("not-a-jwt");

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("auth.invalid_token_header"),);
    }

    #[test]
    fn should_reject_token_when_missing_kid() {
        // A JWT with no "kid" in the header
        // Header: {"alg":"RS256","typ":"JWT"} (no kid)
        // Payload: {"sub":"123"}
        let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjMifQ.fake-signature";

        let result = extract_uid_from_token(token);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("auth.missing_kid"));
    }

    #[test]
    fn should_reject_token_when_kid_not_in_cache() {
        // Header: {"alg":"RS256","typ":"JWT","kid":"unknown-kid"}
        // Base64url: eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6InVua25vd24ta2lkIn0
        let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6InVua25vd24ta2lkIn0.eyJzdWIiOiIxMjMifQ.fake-signature";

        // Set up empty cache
        {
            let mut cache = CERTS_CACHE.write().unwrap();
            *cache = Some(CachedCerts {
                keys: HashMap::new(),
                fetched_at: Instant::now(),
            });
        }

        let result = extract_uid_from_token(token);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("auth.unknown_kid"));
    }
}
