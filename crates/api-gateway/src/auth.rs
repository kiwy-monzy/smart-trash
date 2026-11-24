//! Sign-in and session checks for the gateway.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use rpc_common::auth_token::{self, TokenError};
use rpc_common::authentication::{Caller, Role};

/// Role of a demo user (shared with the gRPC login service).
pub use rpc_common::demo::demo_role;

#[derive(Clone)]
pub struct AuthConfig {
    secret: Arc<String>,
    pub demo_login: bool,
    pub ttl_seconds: i64,
}

impl AuthConfig {
    /// Read the auth settings from the environment.
    pub fn from_env() -> Result<Self, String> {
        let secret = auth_token::secret_from_env().map_err(|err| err.to_string())?;
        let demo_login = matches!(
            std::env::var("AUTH_DEMO_LOGIN").unwrap_or_default().trim(),
            "1" | "true" | "TRUE" | "True"
        );
        let ttl_seconds = std::env::var("AUTH_TOKEN_TTL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|ttl| *ttl > 0)
            .unwrap_or(auth_token::DEFAULT_TTL_SECONDS)
            .min(auth_token::MAX_TTL_SECONDS);

        Ok(Self {
            secret: Arc::new(secret),
            demo_login,
            ttl_seconds,
        })
    }

    pub fn issue(&self, user_id: &str, role: Role) -> Result<(String, i64), TokenError> {
        let token = auth_token::issue(&self.secret, user_id, role, self.ttl_seconds)?;
        Ok((token, unix_now() + self.ttl_seconds))
    }

    /// Value sent to the RPC layer, unchanged from what the client sent.
    pub fn forward_value(&self, token: &str) -> String {
        format!("Bearer {token}")
    }
}

/// A verified request: the caller and the token to forward.
#[derive(Clone, Debug)]
pub struct Session {
    pub caller: Caller,
    pub token: String,
    pub expires_at_unix: i64,
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

fn bearer(headers: &axum::http::HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ").map(str::trim)?;
    (!token.is_empty()).then_some(token)
}

fn unauthorized(message: &str) -> Response {
    let mut response = (StatusCode::UNAUTHORIZED, message.to_string()).into_response();
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Bearer"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// Every `/api/…` route except the login one needs a valid token.
pub async fn require_bearer(
    State(config): State<AuthConfig>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    let protected = path.starts_with("/api/") && path != "/api/auth/login";
    if !protected {
        return next.run(request).await;
    }

    let Some(token) = bearer(request.headers()).map(str::to_string) else {
        return unauthorized("missing authorization bearer token");
    };

    match auth_token::verify(&config.secret, &token) {
        Ok(claims) => {
            request.extensions_mut().insert(Session {
                caller: Caller::new(claims.user_id, claims.role),
                token,
                expires_at_unix: claims.expires_at_unix,
            });
            next.run(request).await
        }
        Err(reason) => {
            tracing::warn!(path = %path, reason = %reason, "rejected HTTP request");
            unauthorized("invalid or expired session")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    fn headers_with(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, value.parse().unwrap());
        headers
    }

    #[test]
    fn bearer_tokens_are_parsed_strictly() {
        assert_eq!(bearer(&headers_with("Bearer abc")), Some("abc"));
        assert_eq!(bearer(&headers_with("bearer abc")), None);
        assert_eq!(bearer(&headers_with("Bearer   ")), None);
        assert_eq!(bearer(&headers_with("Basic abc")), None);
        assert_eq!(bearer(&HeaderMap::new()), None);
    }

    #[test]
    fn the_four_demo_users_have_the_report_roles() {
        assert_eq!(demo_role("HOUSEHOLD-0421"), Some(Role::Household));
        assert_eq!(demo_role("BUSINESS-12"), Some(Role::Business));
        assert_eq!(demo_role("DRIVER-07"), Some(Role::Driver));
        assert_eq!(demo_role("COUNCIL-01"), Some(Role::Council));
        assert_eq!(demo_role("HOUSEHOLD-9999"), None);
    }
    #[test]
    fn session_tokens_carry_their_own_expiry() {
        let config = AuthConfig {
            secret: Arc::new("test-secret-that-is-long-enough-0123456789".into()),
            demo_login: true,
            ttl_seconds: 60,
        };
        let (token, expires_at_unix) = config.issue("COUNCIL-01", Role::Council).unwrap();
        assert!(expires_at_unix > unix_now());
        assert!(token.starts_with("COUNCIL-01.council."));
    }
}
