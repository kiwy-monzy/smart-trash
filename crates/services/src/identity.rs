//! Identity RPC: users, roles and authorization decisions.

use rpc_common::authentication::{Caller, Role as CallerRole};
use rpc_common::validation;
use smarttrash_proto::identity::identity_service_server::IdentityService;
use smarttrash_proto::identity::{
    AuthorizeRequest, AuthorizeResponse, GetUserRequest, ListUsersRequest, ListUsersResponse, User,
};
use tonic::{Request, Response, Status};

use crate::users::{role_from_proto, UserDirectory};

/// Reads the directory shared with the auth and tracking services.
pub struct IdentityRpc {
    users: UserDirectory,
}

impl Default for IdentityRpc {
    fn default() -> Self {
        Self {
            users: UserDirectory::default(),
        }
    }
}

impl IdentityRpc {
    pub fn new(users: UserDirectory) -> Self {
        Self { users }
    }
}

#[tonic::async_trait]
impl IdentityService for IdentityRpc {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<User>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("identity.read")?;
        let id = request.into_inner().user_id;
        validation::identifier("user_id", &id)?;

        // Own record only, unless you are a council.
        if caller.role != CallerRole::Council && id != caller.user_id {
            return Err(Status::permission_denied(
                "a caller may only read its own user record",
            ));
        }

        let user = self
            .users
            .get(&id)
            .ok_or_else(|| rpc_common::not_found("User", &id))?;
        Ok(Response::new(user))
    }

    async fn authorize(
        &self,
        request: Request<AuthorizeRequest>,
    ) -> Result<Response<AuthorizeResponse>, Status> {
        let caller = Caller::from_request(&request)?;
        let data = request.into_inner();

        if caller.role != CallerRole::Council && data.user_id != caller.user_id {
            return Err(Status::permission_denied(
                "a caller may only authorise its own actions",
            ));
        }
        validation::identifier("user_id", &data.user_id)?;
        validation::text("action", &data.action, 64)?;

        let user = self
            .users
            .get(&data.user_id)
            .ok_or_else(|| rpc_common::not_found("User", &data.user_id))?;

        let caller = Caller::new(&user.user_id, role_from_proto(user.role()));
        let result = caller.require_action(&data.action);

        Ok(Response::new(AuthorizeResponse {
            allowed: result.is_ok(),
            reason: result
                .err()
                .map(|status| status.message().to_string())
                .unwrap_or_default(),
        }))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("identity.list")?;

        let limit = validation::page_size(request.into_inner().page_size, 100, 500) as usize;
        let mut users = self.users.list();
        users.sort_by(|a, b| a.user_id.cmp(&b.user_id));
        users.truncate(limit);

        Ok(Response::new(ListUsersResponse { users }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smarttrash_proto::identity::Role;

    #[test]
    fn the_directory_has_one_seed_user_per_category() {
        let identity = IdentityRpc::default();
        let users = identity.users.list();

        assert_eq!(users.len(), 4);
        for id in ["HOUSEHOLD-0421", "BUSINESS-12", "DRIVER-07", "COUNCIL-01"] {
            assert!(users.iter().any(|u| u.user_id == id), "{id} missing");
        }
    }

    #[test]
    fn household_may_create_but_not_plan_routes() {
        let identity = IdentityRpc::default();
        let household = identity.users.get("HOUSEHOLD-0421").expect("seed");

        let caller = Caller::new(&household.user_id, role_from_proto(household.role()));
        assert_eq!(household.role(), Role::Household);
        assert!(caller.require_action("pickup.create").is_ok());
        assert!(caller.require_action("route.optimize").is_err());
    }
}
