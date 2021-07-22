//! CURRENTLY, THE USERNAME IS THE ID, THIS WILL BE CHANGED WHEN I KNOW HOW TO DO RELATIONAL DB STUFF :)

use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::time;

//===========================================================================//
// PASSWORD HASHING                                                          //
//===========================================================================//

/// Create a hashed and salted, encoded password for storing in a database
pub fn hash_password(pwd: &str) -> String {
    let salt: [u8; 16] = rand::random();
    argon2::hash_encoded(pwd.as_bytes(), &salt, &argon2::Config::default()).unwrap()
}

/// Verify the hashed password
pub fn verify_pwdhash(pwdhash: &str, pwd: &str) -> Result<bool> {
    Ok(argon2::verify_encoded(&pwdhash, pwd.as_bytes())?)
}

//===========================================================================//
// USER LOGIN                                                                //
//===========================================================================//

const JWT_SECRET: &[u8] = b"NEVER-STORE-SECRETS-IN-CODE";
static ENCODING_KEY: Lazy<EncodingKey> = Lazy::new(|| EncodingKey::from_secret(JWT_SECRET));
static DECODING_KEY: Lazy<DecodingKey> = Lazy::new(|| DecodingKey::from_secret(JWT_SECRET));

// A custom payload, I can store arbitary information here
// such as different roles/permissions for different users
// to avoid doing a db lookup of such information
#[derive(Serialize, Deserialize)]
pub struct LoginClaims {
    // subject
    pub sub: String, // todo: be the user_id in the DB
    // issued at
    pub iat: u64,
    // expiration
    pub exp: u64,
}

/// Return a JWT (which is JSON)
///
/// Does no authorisation, just straight-up creation,
/// that we can later verify was un-tampered; when the
/// client gives us a JWT in his Authorization header
pub fn generate_login_token(username: &str) -> Result<String> {
    let now = time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH)?;

    let iat = now.as_secs();

    let exp = iat + 86400; // 86400 secs = 24 hours.

    let claims = LoginClaims {
        sub: String::from(username),
        iat,
        exp,
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
pub fn verify_decode_login_token(token: &str) -> Result<LoginClaims> {
    Ok(decode::<LoginClaims>(&token, &DECODING_KEY, &Validation::default())?.claims)
}
