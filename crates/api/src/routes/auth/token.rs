use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::{rng, RngCore};
use axum::http::StatusCode;

pub struct RefreshTokenParts {
    pub raw_token: String,
    pub selector: String,
    pub verifier_hash: String,
}

pub fn generate_refresh_token_parts() -> Result<RefreshTokenParts, (StatusCode, String)> {
    let mut raw_bytes = [0u8; 32];
    rng().fill_bytes(&mut raw_bytes);

    let selector_bytes = &raw_bytes[..16];
    let verifier_bytes = &raw_bytes[16..];

    let selector = URL_SAFE_NO_PAD.encode(selector_bytes);
    let raw_token = URL_SAFE_NO_PAD.encode(raw_bytes);
    let verifier_hash = hash(verifier_bytes, DEFAULT_COST)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash token".to_string()))?;

    Ok(RefreshTokenParts {
        raw_token,
        selector,
        verifier_hash,
    })
}

pub fn split_refresh_token(
    token: &str,
) -> Result<(String, Vec<u8>), (StatusCode, String)> {
    let bytes = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token format".to_string()))?;

    if bytes.len() != 32 {
        return Err((StatusCode::UNAUTHORIZED, "Invalid token length".to_string()));
    }

    let selector = URL_SAFE_NO_PAD.encode(&bytes[..16]);
    Ok((selector, bytes[16..].to_vec()))
}

pub fn verify_token(verifier_bytes: &[u8], verifier_hash: &str) -> Result<bool, (StatusCode, String)> {
    verify(verifier_bytes, verifier_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Verification failed".to_string()))
}
