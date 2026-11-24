//! Tracking RPC: drivers report GPS positions, dashboards stream them.

use std::collections::HashMap;
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

use rpc_common::authentication::Caller;
use rpc_common::validation;
use smarttrash_proto::tracking::tracking_service_server::TrackingService;
use smarttrash_proto::tracking::{
    GetVehicleLocationRequest, LocationAck, ReportLocationRequest, StreamVehicleLocationsRequest,
    VehicleLocation,
};

use crate::notifications::NotificationStore;
use crate::users::{h3_cell, UserDirectory};

const BROADCAST_CAPACITY: usize = 256;
const MAX_TRACKED_VEHICLES: usize = 10_000;
const MAX_SPEED_MPS: f64 = 100.0;
const MAX_CLOCK_SKEW: i64 = 300;
const MAX_PING_AGE: i64 = 7 * 24 * 60 * 60;

pub struct TrackingRpc {
    positions: Mutex<HashMap<String, VehicleLocation>>,
    updates: broadcast::Sender<VehicleLocation>,
    users: UserDirectory,
    notifications: Arc<NotificationStore>,
}

impl TrackingRpc {
    pub fn new(users: UserDirectory, notifications: Arc<NotificationStore>) -> Self {
        let (updates, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            positions: Mutex::new(HashMap::new()),
            updates,
            users,
            notifications,
        }
    }
}

impl Default for TrackingRpc {
    fn default() -> Self {
        Self::new(UserDirectory::default(), Arc::new(NotificationStore::default()))
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

fn validate(location: &ReportLocationRequest) -> Result<(), Status> {
    validation::identifier("vehicle_id", &location.vehicle_id)?;
    validation::identifier("driver_id", &location.driver_id)?;
    validation::latitude("latitude", location.latitude)?;
    validation::longitude("longitude", location.longitude)?;
    validation::number("speed_mps", location.speed_mps, 0.0, MAX_SPEED_MPS)?;
    validation::number("heading_degrees", location.heading_degrees, 0.0, 360.0)?;

    let now = unix_now();
    if location.recorded_at_unix > 0 {
        if location.recorded_at_unix > now + MAX_CLOCK_SKEW {
            return Err(rpc_common::invalid_argument(
                "recorded_at_unix is too far in the future",
            ));
        }
        if location.recorded_at_unix < now - MAX_PING_AGE {
            return Err(rpc_common::invalid_argument("recorded_at_unix is too old"));
        }
    }
    Ok(())
}

#[tonic::async_trait]
impl TrackingService for TrackingRpc {
    async fn report_location(
        &self,
        request: Request<ReportLocationRequest>,
    ) -> Result<Response<LocationAck>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("tracking.report")?;
        let data = request.into_inner();
        validate(&data)?;

        // A driver moves only its own truck.
        if data.driver_id != caller.user_id {
            return Err(Status::permission_denied(format!(
                "caller '{}' may not report for driver '{}'",
                caller.user_id, data.driver_id
            )));
        }

        let recorded_at_unix = if data.recorded_at_unix > 0 {
            data.recorded_at_unix
        } else {
            unix_now()
        };

        let location = VehicleLocation {
            vehicle_id: data.vehicle_id.clone(),
            latitude: data.latitude,
            longitude: data.longitude,
            speed_mps: data.speed_mps,
            heading_degrees: data.heading_degrees,
            recorded_at_unix,
        };

        {
            let mut positions = rpc_common::lock(&self.positions);
            let known = positions.contains_key(&location.vehicle_id);
            if !known && positions.len() >= MAX_TRACKED_VEHICLES {
                return Err(Status::resource_exhausted(
                    "too many vehicles are being tracked",
                ));
            }
            positions.insert(location.vehicle_id.clone(), location.clone());
        }

        let _ = self.updates.send(location.clone());

        // Tell everyone whose home cell touches this one.
        let cell = h3_cell(location.latitude, location.longitude);
        if !cell.is_empty() {
            self.notifications
                .notify_nearby(&self.users, &location.vehicle_id, &cell);
        }

        Ok(Response::new(LocationAck {
            vehicle_id: data.vehicle_id,
            accepted_at_unix: unix_now(),
        }))
    }

    async fn get_vehicle_location(
        &self,
        request: Request<GetVehicleLocationRequest>,
    ) -> Result<Response<VehicleLocation>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("tracking.view")?;
        let id = request.into_inner().vehicle_id;
        validation::identifier("vehicle_id", &id)?;

        let location = rpc_common::lock(&self.positions)
            .get(&id)
            .cloned()
            .ok_or_else(|| rpc_common::not_found("Vehicle position", &id))?;

        tracing::debug!(vehicle_id = %id, caller = %caller.user_id, "position read");
        Ok(Response::new(location))
    }

    type StreamVehicleLocationsStream =
        Pin<Box<dyn Stream<Item = Result<VehicleLocation, Status>> + Send + 'static>>;

    async fn stream_vehicle_locations(
        &self,
        request: Request<StreamVehicleLocationsRequest>,
    ) -> Result<Response<Self::StreamVehicleLocationsStream>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("tracking.view")?;

        let requested = request.into_inner().vehicle_ids;
        if requested.len() > MAX_TRACKED_VEHICLES {
            return Err(rpc_common::invalid_argument("too many vehicle_ids"));
        }
        let filter: HashSet<String> = requested.into_iter().collect();

        // Current positions, then every update from now on.
        let snapshot: Vec<Result<VehicleLocation, Status>> = rpc_common::lock(&self.positions)
            .values()
            .filter(|location| filter.is_empty() || filter.contains(&location.vehicle_id))
            .cloned()
            .map(Ok)
            .collect();

        let stream_filter = filter.clone();
        let live = BroadcastStream::new(self.updates.subscribe()).filter_map(move |update| {
            match update {
                Ok(location)
                    if stream_filter.is_empty() || stream_filter.contains(&location.vehicle_id) =>
                {
                    Some(Ok(location))
                }
                _ => None,
            }
        });

        let combined: Self::StreamVehicleLocationsStream =
            Box::pin(tokio_stream::iter(snapshot).chain(live));

        tracing::debug!(caller = %caller.user_id, "tracking stream opened");
        Ok(Response::new(combined))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ping(driver: &str, vehicle: &str) -> ReportLocationRequest {
        ReportLocationRequest {
            vehicle_id: vehicle.into(),
            driver_id: driver.into(),
            latitude: -6.7,
            longitude: 39.2,
            speed_mps: 8.0,
            heading_degrees: 90.0,
            recorded_at_unix: 0,
        }
    }

    #[test]
    fn a_normal_ping_validates() {
        assert!(validate(&ping("DRIVER-07", "VEH-002")).is_ok());
    }

    #[test]
    fn impossible_coordinates_and_speeds_are_refused() {
        let mut bad = ping("DRIVER-07", "VEH-002");
        bad.latitude = 123.0;
        assert!(validate(&bad).is_err());

        let mut fast = ping("DRIVER-07", "VEH-002");
        fast.speed_mps = 5_000.0;
        assert!(validate(&fast).is_err());

        let mut bad_heading = ping("DRIVER-07", "VEH-002");
        bad_heading.heading_degrees = 361.0;
        assert!(validate(&bad_heading).is_err());
    }

    #[test]
    fn vehicle_and_driver_ids_are_identifier_shaped() {
        let bad = ping("DRIVER-07", "<script>");
        assert!(validate(&bad).is_err());
    }

    #[test]
    fn future_pings_are_refused() {
        let mut future = ping("DRIVER-07", "VEH-002");
        future.recorded_at_unix = unix_now() + 3_600;
        assert!(validate(&future).is_err());

        let mut stale = ping("DRIVER-07", "VEH-002");
        stale.recorded_at_unix = unix_now() - MAX_PING_AGE - 60;
        assert!(validate(&stale).is_err());
    }
}
