//! One user directory shared by the auth, identity and tracking services.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rpc_common::authentication::Role;
use smarttrash_proto::identity::{Role as ProtoRole, User};

pub const H3_RESOLUTION: h3o::Resolution = h3o::Resolution::Nine;

/// H3 cell of a coordinate, empty when the input is invalid.
pub fn h3_cell(latitude: f64, longitude: f64) -> String {
    h3o::LatLng::new(latitude, longitude)
        .map(|point| point.to_cell(H3_RESOLUTION).to_string())
        .unwrap_or_default()
}

pub fn proto_role(role: Role) -> ProtoRole {
    match role {
        Role::Household => ProtoRole::Household,
        Role::Business => ProtoRole::Business,
        Role::Driver => ProtoRole::Driver,
        Role::Council => ProtoRole::Council,
    }
}

pub fn role_from_proto(role: ProtoRole) -> Role {
    match role {
        ProtoRole::Business => Role::Business,
        ProtoRole::Driver => Role::Driver,
        ProtoRole::Council => Role::Council,
        ProtoRole::Household | ProtoRole::Unspecified => Role::Household,
    }
}

fn id_prefix(role: Role) -> &'static str {
    match role {
        Role::Household => "HOUSEHOLD",
        Role::Business => "BUSINESS",
        Role::Driver => "DRIVER",
        Role::Council => "COUNCIL",
    }
}

/// Demo seeds plus everyone who registered in this run (in memory).
#[derive(Clone)]
pub struct UserDirectory {
    users: Arc<Mutex<HashMap<String, User>>>,
    counter: Arc<Mutex<u32>>,
}

impl Default for UserDirectory {
    fn default() -> Self {
        let seeds = [
            (
                "HOUSEHOLD-0421",
                "Asha Mwinyi",
                "asha@example.com",
                Role::Household,
                "Kinondoni",
                -6.7565,
                39.2049,
            ),
            (
                "BUSINESS-12",
                "Kariakoo Traders",
                "ops@kariakoo.example.com",
                Role::Business,
                "Ilala",
                -6.8200,
                39.2800,
            ),
            (
                "DRIVER-07",
                "Juma Kibona",
                "juma@example.com",
                Role::Driver,
                "Kinondoni",
                -6.7700,
                39.2300,
            ),
            (
                "COUNCIL-01",
                "Kinondoni Council",
                "reporting@kinondoni.go.tz",
                Role::Council,
                "Kinondoni",
                -6.7924,
                39.2083,
            ),
        ];

        let users = seeds
            .into_iter()
            .map(|(id, name, email, role, ward, lat, lng)| {
                (
                    id.to_string(),
                    User {
                        user_id: id.to_string(),
                        display_name: name.to_string(),
                        email: email.to_string(),
                        role: proto_role(role) as i32,
                        ward: ward.to_string(),
                        created_at_unix: 0,
                        latitude: lat,
                        longitude: lng,
                        h3_cell: h3_cell(lat, lng),
                    },
                )
            })
            .collect();

        Self {
            users: Arc::new(Mutex::new(users)),
            counter: Arc::new(Mutex::new(0)),
        }
    }
}

impl UserDirectory {
    pub fn get(&self, user_id: &str) -> Option<User> {
        rpc_common::lock(&self.users).get(user_id).cloned()
    }

    pub fn list(&self) -> Vec<User> {
        rpc_common::lock(&self.users).values().cloned().collect()
    }

    /// Create a user and return it. Ids look like `HOUSEHOLD-0101`.
    pub fn register(
        &self,
        display_name: &str,
        role: Role,
        ward: &str,
        latitude: f64,
        longitude: f64,
    ) -> User {
        let mut counter = rpc_common::lock(&self.counter);
        *counter += 1;
        let user_id = format!("{}-{:04}", id_prefix(role), 100 + *counter);
        drop(counter);

        let user = User {
            user_id: user_id.clone(),
            display_name: display_name.to_string(),
            email: String::new(),
            role: proto_role(role) as i32,
            ward: ward.to_string(),
            created_at_unix: 0,
            latitude,
            longitude,
            h3_cell: h3_cell(latitude, longitude),
        };

        rpc_common::lock(&self.users).insert(user_id, user.clone());
        user
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_is_one_seed_user_per_category() {
        let directory = UserDirectory::default();
        assert_eq!(directory.list().len(), 4);
    }

    #[test]
    fn registration_creates_a_usable_profile() {
        let directory = UserDirectory::default();
        let user = directory.register("Test House", Role::Household, "Kinondoni", -6.78, 39.21);

        assert!(user.user_id.starts_with("HOUSEHOLD-"));
        assert_eq!(user.ward, "Kinondoni");
        assert!(!user.h3_cell.is_empty());
        assert!(directory.get(&user.user_id).is_some());
    }
}
