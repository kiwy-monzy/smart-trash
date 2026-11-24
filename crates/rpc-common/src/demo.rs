//! The four demo users from the report (§7.2), shared by the gateway and the
//! gRPC login service.

use crate::authentication::Role;

pub const DEMO_USERS: [(&str, Role); 4] = [
    ("HOUSEHOLD-0421", Role::Household),
    ("BUSINESS-12", Role::Business),
    ("DRIVER-07", Role::Driver),
    ("COUNCIL-01", Role::Council),
];

/// Role of a demo user, or `None` when the id is not one of the seeds.
pub fn demo_role(user_id: &str) -> Option<Role> {
    DEMO_USERS
        .iter()
        .find(|(id, _)| *id == user_id)
        .map(|(_, role)| *role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_seed_user_per_category() {
        assert_eq!(DEMO_USERS.len(), 4);
        assert_eq!(demo_role("HOUSEHOLD-0421"), Some(Role::Household));
        assert_eq!(demo_role("COUNCIL-99"), None);
    }
}
