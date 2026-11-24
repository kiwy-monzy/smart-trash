//! Input checks shared by the gateway and the RPC services.

use tonic::Status;

use crate::invalid_argument;

/// Longest accepted identifier.
pub const MAX_ID_LEN: usize = 64;

/// Identifier: letters, digits, `_` and `-` only.
pub fn identifier(field: &str, value: &str) -> Result<(), Status> {
    if value.is_empty() {
        return Err(invalid_argument(format!("{field} must not be empty")));
    }
    if value.len() > MAX_ID_LEN {
        return Err(invalid_argument(format!(
            "{field} must be at most {MAX_ID_LEN} characters"
        )));
    }
    if !value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(invalid_argument(format!(
            "{field} may only contain letters, digits, '_' and '-'"
        )));
    }
    Ok(())
}

/// Free text: trimmed, not empty, no control characters.
pub fn text(field: &str, value: &str, max_len: usize) -> Result<(), Status> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(invalid_argument(format!("{field} must not be empty")));
    }
    if trimmed.len() > max_len {
        return Err(invalid_argument(format!(
            "{field} must be at most {max_len} characters"
        )));
    }
    if value.chars().any(|c| c.is_control()) {
        return Err(invalid_argument(format!(
            "{field} must not contain control characters"
        )));
    }
    Ok(())
}

pub fn latitude(field: &str, value: f64) -> Result<(), Status> {
    if !value.is_finite() || !(-90.0..=90.0).contains(&value) {
        return Err(invalid_argument(format!("{field} must be within [-90, 90]")));
    }
    Ok(())
}

pub fn longitude(field: &str, value: f64) -> Result<(), Status> {
    if !value.is_finite() || !(-180.0..=180.0).contains(&value) {
        return Err(invalid_argument(format!(
            "{field} must be within [-180, 180]"
        )));
    }
    Ok(())
}

/// Finite number inside `[min, max]`.
pub fn number(field: &str, value: f64, min: f64, max: f64) -> Result<(), Status> {
    if !value.is_finite() || value < min || value > max {
        return Err(invalid_argument(format!(
            "{field} must be a finite number within [{min}, {max}]"
        )));
    }
    Ok(())
}

/// Empty, or a plain `http(s)://` link.
pub fn evidence_url(value: &str) -> Result<(), Status> {
    if value.is_empty() {
        return Ok(());
    }
    if value.len() > 512 {
        return Err(invalid_argument("evidence_url must be at most 512 characters"));
    }
    let lower = value.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err(invalid_argument(
            "evidence_url must start with http:// or https://",
        ));
    }
    if value.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(invalid_argument("evidence_url must not contain whitespace"));
    }
    Ok(())
}

/// `yyyy-mm`.
pub fn month(value: &str) -> Result<(), Status> {
    let bytes = value.as_bytes();
    let shape_ok = bytes.len() == 7
        && bytes[4] == b'-'
        && bytes[..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..].iter().all(|b| b.is_ascii_digit());
    if !shape_ok {
        return Err(invalid_argument("month must look like yyyy-mm"));
    }
    let month: u32 = value[5..].parse().unwrap_or(0);
    if !(1..=12).contains(&month) {
        return Err(invalid_argument("month must be between 01 and 12"));
    }
    Ok(())
}

/// Clamp `page_size` into `[1, max]`; `<= 0` means "use the default".
pub fn page_size(requested: i32, default: i64, max: i64) -> i64 {
    if requested <= 0 {
        default
    } else {
        i64::from(requested).min(max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_accept_the_project_id_shape() {
        for id in ["HOUSEHOLD-0421", "BUSINESS-12", "PICKUP-00001", "VEH_001"] {
            assert!(identifier("id", id).is_ok(), "{id} should be valid");
        }
    }

    #[test]
    fn identifiers_reject_injection_and_markup() {
        let too_long = "x".repeat(MAX_ID_LEN + 1);
        for id in [
            "PICKUP-1' OR '1'='1",
            "1;DROP TABLE pickups;--",
            "<img src=x onerror=alert(1)>",
            "a\0b",
            "a b",
            "",
            too_long.as_str(),
        ] {
            assert!(identifier("id", id).is_err(), "{id:?} must be refused");
        }
    }

    #[test]
    fn coordinates_reject_nan_and_out_of_range() {
        assert!(latitude("lat", 0.0).is_ok());
        assert!(latitude("lat", 90.0).is_ok());
        assert!(latitude("lat", 90.1).is_err());
        assert!(latitude("lat", f64::NAN).is_err());
        assert!(longitude("lon", 180.0).is_ok());
        assert!(longitude("lon", -180.1).is_err());
        assert!(longitude("lon", f64::INFINITY).is_err());
    }

    #[test]
    fn evidence_urls_must_be_plain_http() {
        assert!(evidence_url("").is_ok());
        assert!(evidence_url("https://cdn.example.com/a.jpg").is_ok());
        assert!(evidence_url("javascript:alert(1)").is_err());
        assert!(evidence_url("data:text/html,<script>").is_err());
        assert!(evidence_url("https://example.com/a b.jpg").is_err());
    }

    #[test]
    fn month_shape_is_enforced() {
        assert!(month("2026-11").is_ok());
        assert!(month("2026-13").is_err());
        assert!(month("2026-1").is_err());
        assert!(month("2026-11' OR 1=1--").is_err());
    }

    #[test]
    fn page_size_is_clamped() {
        assert_eq!(page_size(0, 100, 200), 100);
        assert_eq!(page_size(-5, 100, 200), 100);
        assert_eq!(page_size(50, 100, 200), 50);
        assert_eq!(page_size(999_999_999, 100, 200), 200);
    }
}
