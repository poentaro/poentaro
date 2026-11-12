//! The two things accounts need from WebCrypto (the Workers runtime's
//! `crypto` global): unpredictable random bytes for session tokens and
//! salts (and request trace IDs), and PBKDF2 for password hashes. Runtime-only, so not exercised by
//! host-side `cargo test`; the rules around them live in `crate::auth`.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Crypto, CryptoKey, Pbkdf2Params};
use worker::{js_sys, wasm_bindgen, wasm_bindgen_futures, Result};

use crate::auth::password::HASH_LENGTH;
use crate::auth::session::SESSION_TOKEN_BYTES;

fn crypto() -> Result<Crypto> {
    let crypto = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("crypto"))?;
    Ok(crypto.unchecked_into())
}

/// `length` bytes from `crypto.getRandomValues`, the runtime's CSPRNG.
pub(crate) fn random_bytes(length: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0; length];
    crypto()?.get_random_values_with_u8_array(&mut bytes)?;
    Ok(bytes)
}

/// A new session token: `SESSION_TOKEN_BYTES` random bytes, base64url.
pub(crate) fn new_session_token() -> Result<String> {
    Ok(URL_SAFE_NO_PAD.encode(random_bytes(SESSION_TOKEN_BYTES)?))
}

/// A new API key: `poe_` and `KEY_BYTES` random bytes, base64url.
pub(crate) fn new_api_key() -> Result<String> {
    Ok(format!(
        "{}{}",
        crate::api_keys::KEY_PREFIX,
        URL_SAFE_NO_PAD.encode(random_bytes(crate::api_keys::KEY_BYTES)?)
    ))
}

/// A new `api_keys.id`: 16 random bytes, base64url.
pub(crate) fn new_api_key_id() -> Result<String> {
    Ok(URL_SAFE_NO_PAD.encode(random_bytes(16)?))
}

/// A new `users.id`: 16 random bytes, base64url -- unguessable, like a
/// share id, never sequential.
pub(crate) fn new_user_id() -> Result<String> {
    Ok(URL_SAFE_NO_PAD.encode(random_bytes(16)?))
}

/// A new request trace ID (`app_log::RequestMeta::trace_id`): 8 random
/// bytes as 16 lowercase hex characters -- short enough for a user to copy
/// into a message, unique enough to name one request in 30 days of logs.
pub(crate) fn new_trace_id() -> Result<String> {
    Ok(random_bytes(8)?
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// PBKDF2-HMAC-SHA256 of `password` with `salt`, `HASH_LENGTH` bytes long.
pub(crate) async fn pbkdf2_sha256(password: &str, salt: &[u8], iterations: u32) -> Result<Vec<u8>> {
    let subtle = crypto()?.subtle();
    let password_bytes = js_sys::Uint8Array::from(password.as_bytes());
    let usages = js_sys::Array::of1(&JsValue::from_str("deriveBits"));
    let key: CryptoKey = JsFuture::from(subtle.import_key_with_str(
        "raw",
        &password_bytes,
        "PBKDF2",
        false,
        &usages,
    )?)
    .await?
    .unchecked_into();
    let params = Pbkdf2Params::new(
        "PBKDF2",
        &JsValue::from_str("SHA-256"),
        iterations,
        &js_sys::Uint8Array::from(salt),
    );
    let bits =
        JsFuture::from(subtle.derive_bits_with_object(&params, &key, (HASH_LENGTH * 8) as u32)?)
            .await?;
    Ok(js_sys::Uint8Array::new(&bits).to_vec())
}
