//! Session tokens shared by the gateway and the RPC services.
//!
//! A token looks like `user_id.role.expiry.signature` and is signed with
//! HMAC-SHA256 using AUTH_TOKEN_SECRET.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::authentication::Role;

type HmacSha256 = Hmac<Sha256>;

/// Default lifetime of a token (8 hours).
pub const DEFAULT_TTL_SECONDS: i64 = 8 * 60 * 60;
/// A token can never live longer than 30 days.
pub const MAX_TTL_SECONDS: i64 = 30 * 24 * 60 * 60;
/// Shortest accepted secret.
pub const MIN_SECRET_LEN: usize = 32;
/// Environment variable that holds the signing secret.
pub const SECRET_ENV: &str = "AUTH_TOKEN_SECRET";

const MAX_USER_ID_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenError {
    Malformed,
    BadSignature,
    Expired,
    UnknownRole,
    WeakSecret,
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Malformed => "malformed token",
            Self::BadSignature => "bad token signature",
            Self::Expired => "token expired",
            Self::UnknownRole => "unknown role in token",
            Self::WeakSecret => "AUTH_TOKEN_SECRET is missing or shorter than 32 characters",
        };
        f.write_str(text)
    }
}

impl std::error::Error for TokenError {}

/// What a valid token says about the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claims {
    pub user_id: String,
    pub role: Role,
    pub expires_at_unix: i64,
}

pub fn role_name(role: Role) -> &'static str {
    match role {
        Role::Household => "household",
        Role::Business => "business",
        Role::Driver => "driver",
        Role::Council => "council",
    }
}

pub fn role_from_name(name: &str) -> Option<Role> {
    match name {
        "household" => Some(Role::Household),
        "business" => Some(Role::Business),
        "driver" => Some(Role::Driver),
        "council" => Some(Role::Council),
        _ => None,
    }
}

/// Letters, digits, `_` and `-` only, so the `.` separator stays unambiguous.
pub fn valid_user_id(user_id: &str) -> bool {
    !user_id.is_empty()
        && user_id.len() <= MAX_USER_ID_LEN
        && user_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

fn signature(secret: &str, message: &str) -> Result<[u8; 32], TokenError> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| TokenError::WeakSecret)?;
    mac.update(message.as_bytes());
    let bytes = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn from_hex(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if bytes.is_empty() || !bytes.len().is_multiple_of(2) {
        return None;
    }
    bytes
        .chunks(2)
        .map(|pair| {
            let hi = (pair[0] as char).to_digit(16)?;
            let lo = (pair[1] as char).to_digit(16)?;
            Some((hi * 16 + lo) as u8)
        })
        .collect()
}

/// Sign a new token for `user_id`. The lifetime is clamped to 1..=30 days.
pub fn issue(
    secret: &str,
    user_id: &str,
    role: Role,
    ttl_seconds: i64,
) -> Result<String, TokenError> {
    if secret.len() < MIN_SECRET_LEN {
        return Err(TokenError::WeakSecret);
    }
    if !valid_user_id(user_id) {
        return Err(TokenError::Malformed);
    }

    let ttl = ttl_seconds.clamp(1, MAX_TTL_SECONDS);
    let expires_at_unix = unix_now() + ttl;
    let message = format!("{user_id}.{}.{expires_at_unix}", role_name(role));
    let signature = signature(secret, &message)?;

    Ok(format!("{message}.{}", to_hex(&signature)))
}

/// Check the signature, then the expiry.
pub fn verify(secret: &str, token: &str) -> Result<Claims, TokenError> {
    if secret.len() < MIN_SECRET_LEN {
        return Err(TokenError::WeakSecret);
    }

    let mut parts = token.split('.');
    let user_id = parts.next().ok_or(TokenError::Malformed)?;
    let role = parts.next().ok_or(TokenError::Malformed)?;
    let expires = parts.next().ok_or(TokenError::Malformed)?;
    let signature_hex = parts.next().ok_or(TokenError::Malformed)?;
    if parts.next().is_some() {
        return Err(TokenError::Malformed);
    }
    if !valid_user_id(user_id) {
        return Err(TokenError::Malformed);
    }

    let role = role_from_name(role).ok_or(TokenError::UnknownRole)?;
    let expires_at_unix: i64 = expires.parse().map_err(|_| TokenError::Malformed)?;
    let signature_bytes = from_hex(signature_hex).ok_or(TokenError::Malformed)?;

    let message = format!("{user_id}.{}.{expires_at_unix}", role_name(role));
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| TokenError::WeakSecret)?;
    mac.update(message.as_bytes());
    mac.verify_slice(&signature_bytes)
        .map_err(|_| TokenError::BadSignature)?;

    if expires_at_unix <= unix_now() {
        return Err(TokenError::Expired);
    }

    Ok(Claims {
        user_id: user_id.to_string(),
        role,
        expires_at_unix,
    })
}

/// Read the signing secret from the environment.
pub fn secret_from_env() -> Result<String, TokenError> {
    let secret = std::env::var(SECRET_ENV).map_err(|_| TokenError::WeakSecret)?;
    if secret.len() < MIN_SECRET_LEN {
        return Err(TokenError::WeakSecret);
    }
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-that-is-long-enough-0123456789";

    #[test]
    fn issued_token_round_trips() {
        let token = issue(SECRET, "HOUSEHOLD-0421", Role::Household, 60).expect("issue");
        let claims = verify(SECRET, &token).expect("verify");
        assert_eq!(claims.user_id, "HOUSEHOLD-0421");
        assert_eq!(claims.role, Role::Household);
    }

    #[test]
    fn tampering_with_the_role_is_rejected() {
        let token = issue(SECRET, "HOUSEHOLD-0421", Role::Household, 60).expect("issue");
        let forged = token.replacen(".household.", ".council.", 1);
        assert_eq!(verify(SECRET, &forged), Err(TokenError::BadSignature));
    }

    #[test]
    fn another_secret_cannot_verify() {
        let token = issue(SECRET, "DRIVER-07", Role::Driver, 60).expect("issue");
        let other = "another-secret-that-is-also-long-enough-0000";
        assert_eq!(verify(other, &token), Err(TokenError::BadSignature));
    }

    #[test]
    fn expired_tokens_are_rejected() {
        let token = issue(SECRET, "COUNCIL-01", Role::Council, 1).expect("issue");
        let claims = verify(SECRET, &token).expect("still valid");
        assert!(claims.expires_at_unix <= unix_now() + 1);

        let expired = unix_now() - 5;
        let message = format!("COUNCIL-01.council.{expired}");
        let signature = signature(SECRET, &message).expect("sign");
        let forged = format!("{message}.{}", to_hex(&signature));
        assert_eq!(verify(SECRET, &forged), Err(TokenError::Expired));
    }

    #[test]
    fn malformed_tokens_are_rejected_without_panicking() {
        for token in ["", "a", "a.b", "a.b.c", "a.b.c.d.e", "....", "user..1.00"] {
            assert!(verify(SECRET, token).is_err(), "{token:?} must be rejected");
        }
    }

    #[test]
    fn user_ids_with_separators_or_controls_are_refused() {
        for user_id in ["a.b", "a b", "a\nb", "<script>", ""] {
            assert_eq!(
                issue(SECRET, user_id, Role::Household, 60),
                Err(TokenError::Malformed),
                "{user_id:?} must be refused"
            );
        }
    }

    #[test]
    fn short_secrets_are_refused() {
        assert_eq!(
            issue("short", "HOUSEHOLD-0421", Role::Household, 60),
            Err(TokenError::WeakSecret)
        );
    }

    #[test]
    fn ttl_is_clamped() {
        let token = issue(SECRET, "BUSINESS-12", Role::Business, i64::MAX).expect("issue");
        let claims = verify(SECRET, &token).expect("verify");
        assert!(claims.expires_at_unix <= unix_now() + MAX_TTL_SECONDS + 1);
    }

    #[test]
    fn every_role_round_trips_through_its_wire_name() {
        for role in [Role::Household, Role::Business, Role::Driver, Role::Council] {
            assert_eq!(role_from_name(role_name(role)), Some(role));
        }
        assert_eq!(role_from_name("admin"), None);
    }
}
