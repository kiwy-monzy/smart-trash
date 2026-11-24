//! Pickup RPC: collection requests stored in PostgreSQL with Diesel.

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
use diesel::result::Error as DbError;

use database::models::{NewPickup, PickupRow};
use database::schema::pickups;
use rpc_common::authentication::{Caller, Role};
use rpc_common::validation;
use smarttrash_proto::pickup::pickup_service_server::PickupService;
use smarttrash_proto::pickup::{
    AssignPickupRequest, CompletePickupRequest, CreatePickupRequest, GetPickupRequest,
    ListPickupsRequest, ListPickupsResponse, Pickup, PickupStatus, WasteType,
};
use tonic::{Code, Request, Response, Status};

const DEFAULT_PAGE_SIZE: i64 = 100;
const MAX_PAGE_SIZE: i64 = 200;
const MAX_REQUEST_VOLUME: f64 = 100.0;
const MAX_COLLECTED_VOLUME: f64 = 1_000.0;

/// Which rows a caller may see.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Scope {
    All,
    Customer(String),
    Driver(String),
}

impl Scope {
    fn of(caller: &Caller) -> Self {
        match caller.role {
            Role::Council => Self::All,
            Role::Household | Role::Business => Self::Customer(caller.user_id.clone()),
            Role::Driver => Self::Driver(caller.user_id.clone()),
        }
    }
}

pub struct PickupRpc {
    db: Arc<Mutex<PgConnection>>,
}

impl PickupRpc {
    pub fn new(connection: PgConnection) -> Self {
        Self {
            db: Arc::new(Mutex::new(connection)),
        }
    }

    /// Run a query on a blocking thread so the async runtime stays free.
    async fn with_db<T, F>(&self, work: F) -> Result<T, Status>
    where
        T: Send + 'static,
        F: FnOnce(&mut PgConnection) -> Result<T, Status> + Send + 'static,
    {
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let mut guard = rpc_common::lock(&db);
            work(&mut guard)
        })
        .await
        .unwrap_or_else(|join_error| {
            tracing::error!(error = %join_error, "database worker panicked");
            Err(Status::internal("database worker failed"))
        })
    }
}

/// Database failures are logged in full, the client gets a generic message.
fn db_status(err: DbError) -> Status {
    tracing::error!(error = %err, "database error");
    Status::new(Code::Internal, "database error")
}

fn load_pickup(conn: &mut PgConnection, id: &str) -> Result<Pickup, Status> {
    let row: PickupRow = pickups::table
        .find(id)
        .first(conn)
        .map_err(|err| match err {
            DbError::NotFound => rpc_common::not_found("Pickup", id),
            other => db_status(other),
        })?;
    Ok(row_into_pickup(row))
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

fn status_name(status: PickupStatus) -> &'static str {
    match status {
        PickupStatus::Unspecified | PickupStatus::Pending => "PENDING",
        PickupStatus::Scheduled => "SCHEDULED",
        PickupStatus::Assigned => "ASSIGNED",
        PickupStatus::InProgress => "IN_PROGRESS",
        PickupStatus::Completed => "COMPLETED",
        PickupStatus::Cancelled => "CANCELLED",
    }
}

fn status_from_name(name: &str) -> PickupStatus {
    match name {
        "SCHEDULED" => PickupStatus::Scheduled,
        "ASSIGNED" => PickupStatus::Assigned,
        "IN_PROGRESS" => PickupStatus::InProgress,
        "COMPLETED" => PickupStatus::Completed,
        "CANCELLED" => PickupStatus::Cancelled,
        _ => PickupStatus::Pending,
    }
}

fn waste_type_name(waste: WasteType) -> &'static str {
    match waste {
        WasteType::Organic => "ORGANIC",
        WasteType::Recyclable => "RECYCLABLE",
        WasteType::General => "GENERAL",
        WasteType::Hazardous => "HAZARDOUS",
        WasteType::Construction => "CONSTRUCTION",
        WasteType::Unspecified => "GENERAL",
    }
}

fn waste_type_from_name(name: &str) -> WasteType {
    match name {
        "ORGANIC" => WasteType::Organic,
        "RECYCLABLE" => WasteType::Recyclable,
        "HAZARDOUS" => WasteType::Hazardous,
        "CONSTRUCTION" => WasteType::Construction,
        _ => WasteType::General,
    }
}

/// H3 resolution used for pickups (≈0.1 km² cells).
const H3_RESOLUTION: h3o::Resolution = h3o::Resolution::Nine;

/// H3 cell of a coordinate, empty when the input is invalid.
fn h3_cell(latitude: f64, longitude: f64) -> String {
    h3o::LatLng::new(latitude, longitude)
        .map(|point| point.to_cell(H3_RESOLUTION).to_string())
        .unwrap_or_default()
}

fn row_into_pickup(row: PickupRow) -> Pickup {
    Pickup {
        pickup_id: row.pickup_id,
        customer_id: row.customer_id,
        waste_type: waste_type_from_name(&row.waste_type) as i32,
        latitude: row.latitude,
        longitude: row.longitude,
        estimated_volume: row.estimated_volume,
        collected_volume: row.collected_volume,
        evidence_url: row.evidence_url,
        status: status_from_name(&row.status) as i32,
        vehicle_id: row.vehicle_id,
        driver_id: row.driver_id,
        created_at_unix: row.created_at_unix,
        updated_at_unix: row.updated_at_unix,
        h3_cell: row.h3_cell,
        ward: row.ward,
    }
}

#[tonic::async_trait]
impl PickupService for PickupRpc {
    async fn create_pickup(
        &self,
        request: Request<CreatePickupRequest>,
    ) -> Result<Response<Pickup>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("pickup.create")?;
        let data = request.into_inner();

        validation::identifier("customer_id", &data.customer_id)?;
        if data.waste_type() == WasteType::Unspecified {
            return Err(rpc_common::invalid_argument("waste_type must be set"));
        }
        validation::number(
            "estimated_volume",
            data.estimated_volume,
            0.0,
            MAX_REQUEST_VOLUME,
        )?;
        if data.estimated_volume <= 0.0 {
            return Err(rpc_common::invalid_argument(
                "estimated_volume must be > 0",
            ));
        }
        validation::latitude("latitude", data.latitude)?;
        validation::longitude("longitude", data.longitude)?;

        if caller.role != Role::Council && data.customer_id != caller.user_id {
            return Err(Status::permission_denied(format!(
                "caller '{}' may not create pickups for '{}'",
                caller.user_id, data.customer_id
            )));
        }

        let now = unix_now();
        let waste_type = waste_type_name(data.waste_type()).to_string();
        let cell = h3_cell(data.latitude, data.longitude);
        let new_row = NewPickup {
            customer_id: data.customer_id,
            waste_type,
            latitude: data.latitude,
            longitude: data.longitude,
            estimated_volume: data.estimated_volume,
            status: "PENDING".to_string(),
            created_at_unix: now,
            updated_at_unix: now,
            h3_cell: cell,
            ward: String::new(),
        };

        let pickup = self
            .with_db(move |conn| {
                let id: String = diesel::insert_into(pickups::table)
                    .values(&new_row)
                    .returning(pickups::pickup_id)
                    .get_result(conn)
                    .map_err(db_status)?;
                load_pickup(conn, &id)
            })
            .await?;

        tracing::info!(
            pickup_id = %pickup.pickup_id,
            caller = %caller.user_id,
            "pickup saved to PostgreSQL"
        );
        Ok(Response::new(pickup))
    }

    async fn get_pickup(
        &self,
        request: Request<GetPickupRequest>,
    ) -> Result<Response<Pickup>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("pickup.get")?;
        let id = request.into_inner().pickup_id;
        validation::identifier("pickup_id", &id)?;

        let pickup = self.with_db(move |conn| load_pickup(conn, &id)).await?;

        // Row ownership.
        match Scope::of(&caller) {
            Scope::All => {}
            Scope::Customer(owner) if pickup.customer_id == owner => {}
            Scope::Driver(driver) if pickup.driver_id == driver => {}
            _ => {
                return Err(Status::permission_denied(
                    "this pickup belongs to someone else",
                ))
            }
        }

        Ok(Response::new(pickup))
    }

    async fn list_pickups(
        &self,
        request: Request<ListPickupsRequest>,
    ) -> Result<Response<ListPickupsResponse>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("pickup.list")?;
        let data = request.into_inner();

        let filter = match data.status_filter() {
            PickupStatus::Unspecified => None,
            other => Some(status_name(other).to_string()),
        };
        let limit = validation::page_size(data.page_size, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE);
        let scope = Scope::of(&caller);

        let rows: Vec<PickupRow> = self
            .with_db(move |conn| {
                let mut query = pickups::table.into_boxed();
                if let Some(status_filter) = filter {
                    query = query.filter(pickups::status.eq(status_filter));
                }
                query = match scope {
                    Scope::All => query,
                    Scope::Customer(customer) => {
                        query.filter(pickups::customer_id.eq(customer))
                    }
                    Scope::Driver(driver) => query.filter(pickups::driver_id.eq(driver)),
                };
                query
                    .order(pickups::pickup_id.asc())
                    .limit(limit)
                    .load(conn)
                    .map_err(db_status)
            })
            .await?;

        let count = rows.len();
        let pickups = rows.into_iter().map(row_into_pickup).collect();
        tracing::debug!(caller = %caller.user_id, count, "pickups listed");
        Ok(Response::new(ListPickupsResponse { pickups }))
    }

    async fn assign_pickup(
        &self,
        request: Request<AssignPickupRequest>,
    ) -> Result<Response<Pickup>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("pickup.assign")?;
        let data = request.into_inner();

        validation::identifier("pickup_id", &data.pickup_id)?;
        validation::identifier("vehicle_id", &data.vehicle_id)?;
        validation::identifier("driver_id", &data.driver_id)?;

        let pickup = self
            .with_db(move |conn| {
                let current: String = pickups::table
                    .find(&data.pickup_id)
                    .select(pickups::status)
                    .first(conn)
                    .map_err(|err| match err {
                        DbError::NotFound => rpc_common::not_found("Pickup", &data.pickup_id),
                        other => db_status(other),
                    })?;

                if current != "PENDING" && current != "SCHEDULED" {
                    return Err(rpc_common::failed_precondition(format!(
                        "pickup '{}' is {current}, expected PENDING or SCHEDULED",
                        data.pickup_id
                    )));
                }

                diesel::update(pickups::table.find(&data.pickup_id))
                    .set((
                        pickups::status.eq("ASSIGNED"),
                        pickups::vehicle_id.eq(&data.vehicle_id),
                        pickups::driver_id.eq(&data.driver_id),
                        pickups::updated_at_unix.eq(unix_now()),
                    ))
                    .execute(conn)
                    .map_err(db_status)?;

                load_pickup(conn, &data.pickup_id)
            })
            .await?;

        tracing::info!(
            pickup_id = %pickup.pickup_id,
            vehicle = %pickup.vehicle_id,
            caller = %caller.user_id,
            "pickup assigned"
        );
        Ok(Response::new(pickup))
    }

    async fn complete_pickup(
        &self,
        request: Request<CompletePickupRequest>,
    ) -> Result<Response<Pickup>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("pickup.complete")?;
        let data = request.into_inner();

        validation::identifier("pickup_id", &data.pickup_id)?;
        validation::number(
            "collected_volume",
            data.collected_volume,
            0.0,
            MAX_COLLECTED_VOLUME,
        )?;
        validation::evidence_url(&data.evidence_url)?;

        let driver_scope = if caller.role == Role::Driver {
            Some(caller.user_id.clone())
        } else {
            None
        };

        let pickup = self
            .with_db(move |conn| {
                let (current, assigned_driver): (String, String) = pickups::table
                    .find(&data.pickup_id)
                    .select((pickups::status, pickups::driver_id))
                    .first(conn)
                    .map_err(|err| match err {
                        DbError::NotFound => rpc_common::not_found("Pickup", &data.pickup_id),
                        other => db_status(other),
                    })?;

                if let Some(driver) = driver_scope {
                    if assigned_driver != driver {
                        return Err(Status::permission_denied(
                            "this pickup is assigned to another driver",
                        ));
                    }
                }

                if current != "ASSIGNED" && current != "IN_PROGRESS" {
                    return Err(rpc_common::failed_precondition(format!(
                        "pickup '{}' is {current}, expected ASSIGNED or IN_PROGRESS",
                        data.pickup_id
                    )));
                }

                diesel::update(pickups::table.find(&data.pickup_id))
                    .set((
                        pickups::status.eq("COMPLETED"),
                        pickups::collected_volume.eq(data.collected_volume),
                        pickups::evidence_url.eq(&data.evidence_url),
                        pickups::updated_at_unix.eq(unix_now()),
                    ))
                    .execute(conn)
                    .map_err(db_status)?;

                load_pickup(conn, &data.pickup_id)
            })
            .await?;

        tracing::info!(
            pickup_id = %pickup.pickup_id,
            caller = %caller.user_id,
            "pickup completed"
        );
        Ok(Response::new(pickup))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_round_trip() {
        for status in [
            PickupStatus::Pending,
            PickupStatus::Assigned,
            PickupStatus::Completed,
            PickupStatus::Cancelled,
        ] {
            assert_eq!(status_from_name(status_name(status)), status);
        }
    }

    #[test]
    fn waste_type_round_trip() {
        for waste in [
            WasteType::Organic,
            WasteType::Recyclable,
            WasteType::Hazardous,
            WasteType::Construction,
        ] {
            assert_eq!(waste_type_from_name(waste_type_name(waste)), waste);
        }
    }

    #[test]
    fn unknown_database_values_fall_back_safely() {
        assert_eq!(status_from_name("SOMETHING_ELSE"), PickupStatus::Pending);
        assert_eq!(waste_type_from_name(""), WasteType::General);
    }

    #[test]
    fn households_only_see_their_own_rows() {
        let scope = Scope::of(&Caller::new("HOUSEHOLD-0421", Role::Household));
        assert_eq!(scope, Scope::Customer("HOUSEHOLD-0421".into()));
    }

    #[test]
    fn drivers_only_see_rows_assigned_to_them() {
        let scope = Scope::of(&Caller::new("DRIVER-07", Role::Driver));
        assert_eq!(scope, Scope::Driver("DRIVER-07".into()));
    }

    #[test]
    fn councils_see_everything() {
        let scope = Scope::of(&Caller::new("COUNCIL-01", Role::Council));
        assert_eq!(scope, Scope::All);
    }

    #[test]
    fn page_size_defaults_and_is_capped() {
        assert_eq!(
            validation::page_size(0, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE),
            100
        );
        assert_eq!(
            validation::page_size(1_000_000, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE),
            MAX_PAGE_SIZE
        );
    }
}
