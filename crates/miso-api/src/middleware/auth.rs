//! Authentication middleware.

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::ApiError;

/// JWT claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// User role
    pub role: String,
    /// Expiration timestamp
    pub exp: usize,
    /// Issued at timestamp
    pub iat: usize,
}

/// Authenticated user extracted from JWT.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub role: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Get the Authorization header
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        // Check for Bearer token
        if !auth_header.starts_with("Bearer ") {
            return Err(ApiError::Unauthorized);
        }

        let token = &auth_header[7..];

        // Get the JWT secret from environment (in production, this should come from state)
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

        // Decode and validate the token
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| ApiError::Unauthorized)?;

        Ok(AuthUser {
            id: token_data.claims.sub,
            username: token_data.claims.username,
            role: token_data.claims.role,
        })
    }
}

impl AuthUser {
    /// Returns true if the user has admin role.
    pub fn is_admin(&self) -> bool {
        self.role == "admin" || self.role == "super_admin"
    }

    /// Returns true if the user can edit data.
    pub fn can_edit(&self) -> bool {
        matches!(
            self.role.as_str(),
            "technician" | "lab_manager" | "admin" | "super_admin"
        )
    }

    /// Returns true if the user can delete data.
    pub fn can_delete(&self) -> bool {
        matches!(self.role.as_str(), "lab_manager" | "admin" | "super_admin")
    }
}

/// Creates a JWT token for a user.
pub fn create_token(
    user_id: &str,
    username: &str,
    role: &str,
    secret: &str,
    expiration_hours: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, EncodingKey, Header};

    let now = Utc::now();
    let exp = (now + Duration::hours(expiration_hours as i64)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        role: role.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

