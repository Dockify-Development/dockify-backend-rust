/*
    This source file is a part of Dockify
    Dockify is licensed under the Server Side Public License (SSPL), Version 1.
    Find the LICENSE file in the root of this repository for more details.
*/

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use dotenvy::var;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

static JWT_KEY: Lazy<String> = Lazy::new(|| var("JWT_KEY").expect("Failed to retrieve JWT_KEY"));

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject, can be a user identifier
    pub exp: usize,  // Expiration time as a timestamp
}

pub fn generate_jwt(
    key: impl Into<String>,
    exp: DateTime<Utc>,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: key.into().to_owned(), // or whatever you want to use as the subject
        exp: exp.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&JWT_KEY.clone().into_bytes()),
    )
}

pub async fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(&JWT_KEY.clone().into_bytes()),
        &Validation::default(),
    )
    .map(|token_data| token_data.claims)
}
pub fn hash_password(password: impl Into<String>) -> Result<String, argon2::password_hash::Error> {
    let params = Params::new(16, 1, 1, Some(32)).expect("Params error");
    return Ok(Argon2::new(Algorithm::Argon2id, Version::default(), params)
        .hash_password(
            password.into().as_bytes(),
            &SaltString::generate(&mut rand::thread_rng()),
        )?
        .to_string());
}
pub fn verify_password(
    password: &str,
    hashed_password: &str,
) -> Result<(), argon2::password_hash::Error> {
    Argon2::default()
        .verify_password(password.as_bytes(), &PasswordHash::new(hashed_password)?)
        .map_err(|e| {
            tracing::error!("Error verifying password: {}", e);
            e
        })
}

pub async fn validate_token(token: impl Into<String>) -> (bool, String) {
    match verify_jwt(&token.into()).await {
        Ok(claims) => (true, claims.sub),
        Err(_) => (false, "".to_string()),
    }
}

pub async fn validate_request(headers: &HeaderMap) -> (bool, String) {
    let auth_header = match headers.get(axum::http::header::AUTHORIZATION) {
        Some(val) => val,
        None => return (false, "".to_string()),
    };
    let auth_str = match auth_header.to_str() {
        Ok(str) => str,
        Err(_) => return (false, "".to_string()),
    };

    // Bearer <token>, split to remove the Bearer
    let parsed_token = match auth_str.split(' ').nth(1) {
        Some(token) => token,
        None => return (false, "".to_string()),
    };

    validate_token(parsed_token).await
}
static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\w\.-]+@[a-zA-Z\d\.-]+\.[a-zA-Z]{2,}$").unwrap());
static ALLOWED_EMAIL_DOMAINS: Lazy<Vec<&str>> =
    Lazy::new(|| ["gmail.com", "outlook.com", "sigma.town"].to_vec());
pub fn validate_email(email: &str) -> bool {
    if !&email.is_ascii() || !EMAIL_REGEX.is_match(&email) || !is_domain_accepted(email) {
        false
    } else {
        true
    }
}
fn is_domain_accepted(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let domain = parts[1];

    ALLOWED_EMAIL_DOMAINS.contains(&domain)
}
