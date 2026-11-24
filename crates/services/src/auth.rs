//! Authentication interceptor for every RPC call.

use rpc_common::auth_token;
use rpc_common::authentication::Caller;
use tonic::{Request, Status};

fn bearer_token<T>(request: &Request<T>) -> Option<String> {
    let value = request.metadata().get("authorization")?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ").map(str::trim)?;
    (!token.is_empty()).then(|| token.to_string())
}

/// Verify the token and attach the caller to the request.
pub fn authenticate(secret: &str, mut request: Request<()>) -> Result<Request<()>, Status> {
    let token = bearer_token(&request)
        .ok_or_else(|| Status::unauthenticated("missing authorization bearer token"))?;

    let claims = auth_token::verify(secret, &token).map_err(|reason| {
        tracing::warn!(reason = %reason, "rejected RPC call");
        Status::unauthenticated("invalid or expired credentials")
    })?;

    request
        .extensions_mut()
        .insert(Caller::new(claims.user_id, claims.role));
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rpc_common::authentication::Role;
    use tonic::metadata::MetadataValue;
    use tonic::Code;

    const SECRET: &str = "test-secret-that-is-long-enough-0123456789";

    fn request_with(token: Option<&str>) -> Request<()> {
        let mut request = Request::new(());
        if let Some(token) = token {
            request
                .metadata_mut()
                .insert("authorization", MetadataValue::try_from(token).unwrap());
        }
        request
    }

    #[test]
    fn a_valid_token_attaches_the_caller() {
        let token = auth_token::issue(SECRET, "DRIVER-07", Role::Driver, 60).unwrap();
        let request = authenticate(SECRET, request_with(Some(&format!("Bearer {token}"))))
            .expect("valid token accepted");

        let caller = request.extensions().get::<Caller>().expect("caller attached");
        assert_eq!(caller.user_id, "DRIVER-07");
        assert_eq!(caller.role, Role::Driver);
    }

    #[test]
    fn a_missing_token_is_unauthenticated() {
        let status = authenticate(SECRET, request_with(None)).expect_err("must fail");
        assert_eq!(status.code(), Code::Unauthenticated);
    }

    #[test]
    fn a_forged_token_is_unauthenticated() {
        let token = auth_token::issue(SECRET, "HOUSEHOLD-0421", Role::Household, 60).unwrap();
        let forged = format!("Bearer {}", token.replacen(".household.", ".council.", 1));
        let status = authenticate(SECRET, request_with(Some(&forged))).expect_err("must fail");
        assert_eq!(status.code(), Code::Unauthenticated);
    }

    #[test]
    fn a_non_bearer_scheme_is_rejected() {
        let token = auth_token::issue(SECRET, "COUNCIL-01", Role::Council, 60).unwrap();
        let status = authenticate(SECRET, request_with(Some(&format!("Basic {token}"))))
            .expect_err("must fail");
        assert_eq!(status.code(), Code::Unauthenticated);
    }
}
