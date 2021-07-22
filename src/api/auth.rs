//! CURRENTLY, THE USERNAME IS THE ID, THIS WILL BE CHANGED WHEN I KNOW HOW TO DO RELATIONAL DB STUFF :)

use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::time;

const JWT_SECRET: &[u8] = b"NEVER-STORE-SECRETS-IN-CODE";
static ENCODING_KEY: Lazy<EncodingKey> = Lazy::new(|| EncodingKey::from_secret(JWT_SECRET));
static DECODING_KEY: Lazy<DecodingKey> = Lazy::new(|| DecodingKey::from_secret(JWT_SECRET));

// A custom payload, I can store arbitary information here
// such as different roles/permissions for different users
#[derive(Serialize, Deserialize)]
pub struct Claims {
    // subject
    pub sub: String,
    // issued at
    pub iat: u64,
}

/// Return a JWT (which is JSON)
///
/// Does no authorisation, just straight-up creation,
/// that we can later verify was un-tampered; when the
/// client gives us a JWT in his Authorization header
pub fn generate_jwt(username: &str) -> Result<String> {
    let unix_time = time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)?
        .as_secs();

    let claims = Claims {
        sub: String::from(username),
        iat: unix_time,
    };

    Ok(encode(&Header::default(), &claims, &ENCODING_KEY)?)
}

/// Validate the JWT.
///
/// If the token or its signature is invalid or the claims fail validation,
/// it will return an error.
///
/// If Ok(Claims), then you can trust the data, and/or perform additional conditional logic
/// such as check for certain permissions/roles
pub fn verify_decode_jwt(jwt: &str) -> Result<Claims> {
    Ok(decode::<Claims>(
        &jwt,
        &DECODING_KEY,
        &Validation {
            validate_exp: false,
            ..Default::default()
        },
    )?
    .claims)
}
