//! Auth RPC: login and registration, both without a token.

use std::time::{SystemTime, UNIX_EPOCH};

use rpc_common::auth_token;
use rpc_common::validation;
use smarttrash_proto::auth::auth_service_server::AuthService;
use smarttrash_proto::auth::{LoginRequest, LoginResponse, RegisterRequest};
use smarttrash_proto::identity::Role as ProtoRole;
use tonic::{Request, Response, Status};

use crate::users::{proto_role, role_from_proto, UserDirectory};

pub struct AuthRpc {
    secret: String,
    ttl_seconds: i64,
    users: UserDirectory,
}

impl AuthRpc {
    pub fn new(secret: String, ttl_seconds: i64, users: UserDirectory) -> Self {
        Self {
            secret,
            ttl_seconds,
            users,
        }
    }

    fn issue(
        &self,
        user_id: &str,
        role: rpc_common::authentication::Role,
    ) -> Result<LoginResponse, Status> {
        let token = auth_token::issue(&self.secret, user_id, role, self.ttl_seconds).map_err(
            |err| {
                tracing::error!(error = %err, "could not issue a session token");
                Status::internal("could not issue a session")
            },
        )?;

        Ok(LoginResponse {
            token,
            user_id: user_id.to_string(),
            role: proto_role(role) as i32,
            expires_at_unix: unix_now() + self.ttl_seconds,
        })
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

#[tonic::async_trait]
impl AuthService for AuthRpc {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let user_id = request.into_inner().user_id;
        validation::identifier("user_id", &user_id)?;

        // Same answer for every unknown id: no user enumeration.
        let user = self
            .users
            .get(&user_id)
            .ok_or_else(|| Status::unauthenticated("unknown user"))?;

        let role = role_from_proto(user.role());
        tracing::info!(user_id = %user_id, "session issued over gRPC");

        Ok(Response::new(self.issue(&user.user_id, role)?))
    }

    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let data = request.into_inner();

        validation::text("display_name", &data.display_name, 80)?;
        validation::text("ward", &data.ward, 64)?;
        validation::latitude("latitude", data.latitude)?;
        validation::longitude("longitude", data.longitude)?;

        let role = match data.role() {
            ProtoRole::Unspecified => rpc_common::authentication::Role::Household,
            other => role_from_proto(other),
        };

        let user = self.users.register(
            data.display_name.trim(),
            role,
            data.ward.trim(),
            data.latitude,
            data.longitude,
        );

        tracing::info!(user_id = %user.user_id, role = ?role, "account registered");
        Ok(Response::new(self.issue(&user.user_id, role)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-that-is-long-enough-0123456789";

    fn service() -> AuthRpc {
        AuthRpc::new(SECRET.to_string(), 60, UserDirectory::default())
    }

    #[tokio::test]
    async fn a_seed_user_gets_a_usable_token() {
        let response = service()
            .login(Request::new(LoginRequest {
                user_id: "DRIVER-07".into(),
            }))
            .await
            .expect("login works")
            .into_inner();

        assert_eq!(response.role(), ProtoRole::Driver);
        assert!(auth_token::verify(SECRET, &response.token).is_ok());
    }

    #[tokio::test]
    async fn an_unknown_user_is_refused() {
        let status = service()
            .login(Request::new(LoginRequest {
                user_id: "HOUSEHOLD-9999".into(),
            }))
            .await
            .expect_err("must fail");
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn registration_returns_a_session_for_the_new_user() {
        let response = service()
            .register(Request::new(RegisterRequest {
                display_name: "Mama Njeri".into(),
                role: ProtoRole::Household as i32,
                ward: "Kinondoni".into(),
                latitude: -6.78,
                longitude: 39.21,
            }))
            .await
            .expect("register works")
            .into_inner();

        assert!(response.user_id.starts_with("HOUSEHOLD-"));
        assert!(auth_token::verify(SECRET, &response.token).is_ok());
    }

    #[tokio::test]
    async fn registration_validates_the_ward() {
        let status = service()
            .register(Request::new(RegisterRequest {
                display_name: "X".into(),
                role: ProtoRole::Household as i32,
                ward: String::new(),
                latitude: -6.78,
                longitude: 39.21,
            }))
            .await
            .expect_err("must fail");
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }
}
