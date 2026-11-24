//! Fleet RPC: trucks, drivers, vehicle state and assignments.

use std::collections::HashMap;
use std::sync::Mutex;

use rpc_common::authentication::{Caller, Role};
use rpc_common::validation;
use smarttrash_proto::fleet::fleet_service_server::FleetService;
use smarttrash_proto::fleet::{
    AssignDriverRequest, GetVehicleRequest, ListVehiclesRequest, ListVehiclesResponse,
    SetVehicleStateRequest, Vehicle, VehicleAssignment, VehicleState, VehicleType,
};
use tonic::{Request, Response, Status};

const H3_RESOLUTION: h3o::Resolution = h3o::Resolution::Nine;

fn h3_cell(latitude: f64, longitude: f64) -> String {
    h3o::LatLng::new(latitude, longitude)
        .map(|point| point.to_cell(H3_RESOLUTION).to_string())
        .unwrap_or_default()
}

/// In-memory fleet; live positions come from Tracking, not from here.
pub struct FleetRpc {
    vehicles: Mutex<HashMap<String, Vehicle>>,
}

impl Default for FleetRpc {
    fn default() -> Self {
        let mut seed = [
            Vehicle {
                vehicle_id: "VEH-001".into(),
                registration_plate: "T 421 ABA".into(),
                vehicle_type: VehicleType::StandardTruck as i32,
                state: VehicleState::Available as i32,
                capacity_cubic_metres: 12.0,
                driver_id: String::new(),
                latitude: -6.7924,
                longitude: 39.2083,
                activated: false,
                h3_cell: String::new(),
            },
            Vehicle {
                vehicle_id: "VEH-002".into(),
                registration_plate: "T 998 CDX".into(),
                vehicle_type: VehicleType::CompactorTruck as i32,
                state: VehicleState::OnRoute as i32,
                capacity_cubic_metres: 18.0,
                driver_id: "DRIVER-07".into(),
                latitude: -6.7565,
                longitude: 39.2049,
                activated: true,
                h3_cell: String::new(),
            },
            Vehicle {
                vehicle_id: "VEH-003".into(),
                registration_plate: "T 117 EFG".into(),
                vehicle_type: VehicleType::CompactTruck as i32,
                state: VehicleState::Maintenance as i32,
                capacity_cubic_metres: 6.0,
                driver_id: String::new(),
                latitude: -6.8131,
                longitude: 39.2768,
                activated: false,
                h3_cell: String::new(),
            },
        ];

        for vehicle in &mut seed {
            vehicle.h3_cell = h3_cell(vehicle.latitude, vehicle.longitude);
        }

        Self {
            vehicles: Mutex::new(
                seed.into_iter()
                    .map(|vehicle| (vehicle.vehicle_id.clone(), vehicle))
                    .collect(),
            ),
        }
    }
}

#[tonic::async_trait]
impl FleetService for FleetRpc {
    async fn get_vehicle(
        &self,
        request: Request<GetVehicleRequest>,
    ) -> Result<Response<Vehicle>, Status> {
        Caller::from_request(&request)?.require_action("fleet.view")?;
        let id = request.into_inner().vehicle_id;
        validation::identifier("vehicle_id", &id)?;

        let vehicle = rpc_common::lock(&self.vehicles)
            .get(&id)
            .cloned()
            .ok_or_else(|| rpc_common::not_found("Vehicle", &id))?;
        Ok(Response::new(vehicle))
    }

    async fn list_vehicles(
        &self,
        request: Request<ListVehiclesRequest>,
    ) -> Result<Response<ListVehiclesResponse>, Status> {
        Caller::from_request(&request)?.require_action("fleet.view")?;
        let data = request.into_inner();
        let limit = validation::page_size(data.page_size, 50, 200) as usize;

        let mut vehicles: Vec<Vehicle> = rpc_common::lock(&self.vehicles)
            .values()
            .filter(|v| {
                data.state_filter() == VehicleState::Unspecified
                    || v.state() == data.state_filter()
            })
            .cloned()
            .collect();
        vehicles.sort_by(|a, b| a.vehicle_id.cmp(&b.vehicle_id));
        vehicles.truncate(limit);

        Ok(Response::new(ListVehiclesResponse { vehicles }))
    }

    async fn assign_driver(
        &self,
        request: Request<AssignDriverRequest>,
    ) -> Result<Response<VehicleAssignment>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("fleet.assign")?;
        let data = request.into_inner();

        validation::identifier("vehicle_id", &data.vehicle_id)?;
        validation::identifier("driver_id", &data.driver_id)?;
        validation::identifier("route_id", &data.route_id)?;

        let mut vehicles = rpc_common::lock(&self.vehicles);
        let vehicle = vehicles
            .get_mut(&data.vehicle_id)
            .ok_or_else(|| rpc_common::not_found("Vehicle", &data.vehicle_id))?;

        if vehicle.state() != VehicleState::Available && vehicle.state() != VehicleState::Idle {
            return Err(rpc_common::failed_precondition(format!(
                "vehicle '{}' is {:?}, expected AVAILABLE or IDLE",
                data.vehicle_id,
                vehicle.state()
            )));
        }

        vehicle.state = VehicleState::OnRoute as i32;
        vehicle.driver_id = data.driver_id.clone();

        tracing::info!(
            vehicle_id = %data.vehicle_id,
            driver_id = %data.driver_id,
            route_id = %data.route_id,
            caller = %caller.user_id,
            "driver assigned"
        );

        Ok(Response::new(VehicleAssignment {
            vehicle_id: data.vehicle_id,
            driver_id: data.driver_id,
            route_id: data.route_id,
        }))
    }

    /// The driver's on/off switch for the day.
    async fn set_vehicle_state(
        &self,
        request: Request<SetVehicleStateRequest>,
    ) -> Result<Response<Vehicle>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("fleet.activate")?;
        let data = request.into_inner();

        validation::identifier("vehicle_id", &data.vehicle_id)?;
        let new_state = data.state();
        if new_state == VehicleState::Unspecified {
            return Err(rpc_common::invalid_argument("state must be set"));
        }

        let mut vehicles = rpc_common::lock(&self.vehicles);
        let vehicle = vehicles
            .get_mut(&data.vehicle_id)
            .ok_or_else(|| rpc_common::not_found("Vehicle", &data.vehicle_id))?;

        if caller.role == Role::Driver {
            // A driver switches on its own truck, and only on/off.
            if vehicle.driver_id != caller.user_id {
                return Err(Status::permission_denied(
                    "this vehicle is not assigned to you",
                ));
            }
            if new_state != VehicleState::Available && new_state != VehicleState::OutOfService {
                return Err(Status::permission_denied(
                    "a driver may only go AVAILABLE or OUT_OF_SERVICE",
                ));
            }
        }

        vehicle.state = new_state as i32;
        vehicle.activated =
            new_state == VehicleState::Available || new_state == VehicleState::OnRoute;
        vehicle.h3_cell = h3_cell(vehicle.latitude, vehicle.longitude);

        tracing::info!(
            vehicle_id = %vehicle.vehicle_id,
            state = ?new_state,
            activated = vehicle.activated,
            caller = %caller.user_id,
            "vehicle state changed"
        );

        Ok(Response::new(vehicle.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rpc_common::authentication::Caller;

    fn caller(vehicle: &str, user: &str, role: Role) -> (String, Caller) {
        (vehicle.to_string(), Caller::new(user, role))
    }

    #[tokio::test]
    async fn a_driver_cannot_switch_someone_elses_vehicle() {
        let fleet = FleetRpc::default();
        let (vehicle, who) = caller("VEH-001", "DRIVER-07", Role::Driver);

        let mut request = Request::new(SetVehicleStateRequest {
            vehicle_id: vehicle,
            state: VehicleState::Available as i32,
        });
        request.extensions_mut().insert(who);

        let status = fleet.set_vehicle_state(request).await.expect_err("must fail");
        assert_eq!(status.code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn a_driver_can_switch_its_own_vehicle_off() {
        let fleet = FleetRpc::default();
        let (vehicle, who) = caller("VEH-002", "DRIVER-07", Role::Driver);

        let mut request = Request::new(SetVehicleStateRequest {
            vehicle_id: vehicle,
            state: VehicleState::OutOfService as i32,
        });
        request.extensions_mut().insert(who);

        let vehicle = fleet
            .set_vehicle_state(request)
            .await
            .expect("own vehicle")
            .into_inner();
        assert!(!vehicle.activated);
        assert_eq!(vehicle.state(), VehicleState::OutOfService);
    }
}
