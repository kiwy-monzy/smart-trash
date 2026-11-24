//! Who is calling and what that role may do.
//!
//! The four categories come from the report (§7.2): household, business,
//! driver, council. `require_action` is the only place the policy is written.

use tonic::{Code, Request, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Household,
    Business,
    Driver,
    Council,
}

/// The authenticated principal of a request.
#[derive(Debug, Clone)]
pub struct Caller {
    pub user_id: String,
    pub role: Role,
}

impl Caller {
    pub fn new(user_id: impl Into<String>, role: Role) -> Self {
        Self {
            user_id: user_id.into(),
            role,
        }
    }

    /// Caller attached by the authentication interceptor.
    pub fn from_request<T>(request: &Request<T>) -> Result<Caller, Status> {
        request
            .extensions()
            .get::<Caller>()
            .cloned()
            .ok_or_else(missing_identity)
    }

    pub fn require_role(&self, allowed: &[Role]) -> Result<(), Status> {
        if allowed.contains(&self.role) {
            Ok(())
        } else {
            Err(Status::permission_denied(format!(
                "role {:?} may not perform this action",
                self.role
            )))
        }
    }

    /// Permission table: which roles may run `action`.
    pub fn require_action(&self, action: &str) -> Result<(), Status> {
        let allowed: &[Role] = match action {
            // Everyone asks for a collection and follows it.
            "pickup.create" | "pickup.get" | "pickup.list" | "tracking.view"
            | "complaint.create" => &[
                Role::Household,
                Role::Business,
                Role::Driver,
                Role::Council,
            ],

            // Drivers only.
            "tracking.report" => &[Role::Driver],

            // Closing a job: the driver, or the council on its behalf.
            "pickup.complete" => &[Role::Driver, Role::Council],

            // A driver switches its own vehicle on/off; a council can too.
            "fleet.activate" => &[Role::Driver, Role::Council],

            // Recurring pickups and the legal collection records.
            "pickup.recurring" | "collection.record" => &[Role::Business],

            // Planning: routes, trucks, windows, dashboards, reports.
            "pickup.assign"
            | "route.optimize"
            | "fleet.assign"
            | "window.schedule"
            | "dashboard.view"
            | "compliance.report"
            | "identity.list" => &[Role::Council],

            // Read-only views every category needs (row ownership is checked
            // in the handlers).
            "window.get" | "route.get" | "fleet.view" | "identity.read" | "notification.list"
            | "geo.view" => &[
                Role::Household,
                Role::Business,
                Role::Driver,
                Role::Council,
            ],

            // Money.
            "invoice.create" | "invoice.pay" | "invoice.get" => {
                &[Role::Business, Role::Council]
            }

            // Unknown action: nobody.
            _ => &[],
        };

        self.require_role(allowed).map_err(|_| {
            Status::permission_denied(format!(
                "role {:?} may not perform '{action}'",
                self.role
            ))
        })
    }
}

/// Status used when the request carries no identity at all.
pub fn missing_identity() -> Status {
    Status::new(Code::Unauthenticated, "caller identity missing")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn household_can_request_a_pickup() {
        let caller = Caller::new("HOUSEHOLD-0421", Role::Household);
        assert!(caller.require_action("pickup.create").is_ok());
    }

    #[test]
    fn household_can_view_the_truck_and_complain() {
        let caller = Caller::new("HOUSEHOLD-0421", Role::Household);
        assert!(caller.require_action("tracking.view").is_ok());
        assert!(caller.require_action("complaint.create").is_ok());
    }

    #[test]
    fn household_cannot_plan_routes() {
        let caller = Caller::new("HOUSEHOLD-0421", Role::Household);
        assert!(caller.require_action("route.optimize").is_err());
        assert!(caller.require_action("pickup.assign").is_err());
    }

    #[test]
    fn business_can_set_up_recurring_pickups_and_records() {
        let caller = Caller::new("BUSINESS-12", Role::Business);
        assert!(caller.require_action("pickup.recurring").is_ok());
        assert!(caller.require_action("collection.record").is_ok());
    }

    #[test]
    fn driver_reports_status_but_cannot_bill() {
        let caller = Caller::new("DRIVER-07", Role::Driver);
        assert!(caller.require_action("tracking.report").is_ok());
        assert!(caller.require_action("invoice.create").is_err());
    }

    #[test]
    fn council_sees_the_dashboard_and_cannot_report_as_a_driver() {
        let caller = Caller::new("COUNCIL-01", Role::Council);
        assert!(caller.require_action("dashboard.view").is_ok());
        assert!(caller.require_action("compliance.report").is_ok());
        assert!(caller.require_action("tracking.report").is_err());
    }

    #[test]
    fn unknown_action_fails_closed() {
        let caller = Caller::new("COUNCIL-01", Role::Council);
        assert!(caller.require_action("nuclear.launch").is_err());
    }

    #[test]
    fn every_role_can_read_its_own_views() {
        for role in [Role::Household, Role::Business, Role::Driver, Role::Council] {
            let caller = Caller::new("SOMEONE-01", role);
            for action in ["window.get", "route.get", "fleet.view", "identity.read"] {
                assert!(
                    caller.require_action(action).is_ok(),
                    "{role:?} should be allowed to {action}"
                );
            }
        }
    }

    #[test]
    fn only_drivers_and_councils_can_complete_a_pickup() {
        assert!(Caller::new("DRIVER-07", Role::Driver)
            .require_action("pickup.complete")
            .is_ok());
        assert!(Caller::new("COUNCIL-01", Role::Council)
            .require_action("pickup.complete")
            .is_ok());
        assert!(Caller::new("HOUSEHOLD-0421", Role::Household)
            .require_action("pickup.complete")
            .is_err());
        assert!(Caller::new("BUSINESS-12", Role::Business)
            .require_action("pickup.complete")
            .is_err());
    }
}
