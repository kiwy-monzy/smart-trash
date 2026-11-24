//! Consistent `tonic::Status` construction across services.
//!
//! Services return `Status` for failures; keeping the mapping here means the
//! same failure always uses the same gRPC code (and therefore the same HTTP
//! status at the gateway).

use tonic::{Code, Status};

/// A caller sent something the service refuses to process.
pub fn invalid_argument(message: impl Into<String>) -> Status {
    Status::invalid_argument(message.into())
}

/// The requested entity does not exist.
pub fn not_found(entity: &str, id: &str) -> Status {
    Status::new(Code::NotFound, format!("{entity} '{id}' not found"))
}

/// The entity exists but is in the wrong state for this operation.
pub fn failed_precondition(message: impl Into<String>) -> Status {
    Status::failed_precondition(message.into())
}

/// A required caller identity or permission is missing.
pub fn unauthenticated(message: impl Into<String>) -> Status {
    Status::unauthenticated(message.into())
}

/// The service is not ready to handle the request yet.
pub fn unavailable(message: impl Into<String>) -> Status {
    Status::unavailable(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_uses_entity_and_id() {
        let status = not_found("Pickup", "PICKUP-001");
        assert_eq!(status.code(), Code::NotFound);
        assert!(status.message().contains("PICKUP-001"));
    }
}
