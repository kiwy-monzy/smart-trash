//! Routing RPC: nearest-neighbour ordering of pickup stops.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;

use rpc_common::authentication::Caller;
use rpc_common::validation;
use smarttrash_proto::routing::routing_service_server::RoutingService;
use smarttrash_proto::routing::{GetRouteRequest, OptimizeRouteRequest, Route, RouteStatus, Stop};
use tonic::{Request, Response, Status};

/// Depot the routes start and end at (Dar es Salaam).
const DEPOT: (f64, f64) = (-6.7924, 39.2083);
const AVG_SPEED_KMPH: f64 = 25.0;
/// Nearest-neighbour is O(n²), and one truck cannot serve more than this.
const MAX_STOPS: usize = 100;

/// Placeholder coordinates/volume until routing reads the real pickups.
fn fixture(pickup_id: &str) -> (f64, f64, f64) {
    let mut hasher = DefaultHasher::new();
    pickup_id.hash(&mut hasher);
    let hash = hasher.finish();

    let lat_offset = ((hash % 201) as f64 - 100.0) / 1000.0;
    let lon_offset = (((hash >> 8) % 201) as f64 - 100.0) / 1000.0;
    let volume = 1.0 + ((hash >> 16) % 15) as f64 / 10.0;

    (DEPOT.0 + lat_offset, DEPOT.1 + lon_offset, volume)
}

fn haversine_km(a: (f64, f64), b: (f64, f64)) -> f64 {
    const R: f64 = 6371.0;
    let d_lat = (b.0 - a.0).to_radians();
    let d_lon = (b.1 - a.1).to_radians();
    let lat1 = a.0.to_radians();
    let lat2 = b.0.to_radians();

    let h = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * R * h.sqrt().asin()
}

pub struct RoutingRpc {
    routes: Mutex<HashMap<String, Route>>,
    next_id: AtomicI64,
}

impl Default for RoutingRpc {
    fn default() -> Self {
        Self {
            routes: Mutex::new(HashMap::new()),
            next_id: AtomicI64::new(0),
        }
    }
}

#[tonic::async_trait]
impl RoutingService for RoutingRpc {
    async fn optimize_route(
        &self,
        request: Request<OptimizeRouteRequest>,
    ) -> Result<Response<Route>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("route.optimize")?;
        let data = request.into_inner();

        if data.pickup_ids.is_empty() {
            return Err(rpc_common::invalid_argument(
                "pickup_ids must not be empty",
            ));
        }
        if data.pickup_ids.len() > MAX_STOPS {
            return Err(rpc_common::invalid_argument(format!(
                "at most {MAX_STOPS} stops per route"
            )));
        }
        validation::identifier("vehicle_id", &data.vehicle_id)?;
        for pickup_id in &data.pickup_ids {
            validation::identifier("pickup_id", pickup_id)?;
        }
        validation::number(
            "capacity_cubic_metres",
            data.capacity_cubic_metres,
            0.0,
            1_000.0,
        )?;

        let mut remaining: Vec<(f64, f64, f64, String)> = data
            .pickup_ids
            .iter()
            .map(|id| {
                let (lat, lon, volume) = fixture(id);
                (lat, lon, volume, id.clone())
            })
            .collect();

        if data.capacity_cubic_metres > 0.0 {
            let total: f64 = remaining.iter().map(|(_, _, volume, _)| volume).sum();
            if total > data.capacity_cubic_metres {
                return Err(rpc_common::failed_precondition(format!(
                    "requested volume {total:.1} m³ exceeds truck capacity {:.1} m³",
                    data.capacity_cubic_metres
                )));
            }
        }

        let mut stops = Vec::with_capacity(remaining.len());
        let mut current = DEPOT;
        let mut total_distance_km = 0.0;

        while !remaining.is_empty() {
            let nearest = remaining
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let da = haversine_km(current, (a.0, a.1));
                    let db = haversine_km(current, (b.0, b.1));
                    da.total_cmp(&db)
                })
                .map(|(index, _)| index)
                .expect("remaining is not empty");

            let (lat, lon, volume, pickup_id) = remaining.remove(nearest);
            total_distance_km += haversine_km(current, (lat, lon));
            current = (lat, lon);

            stops.push(Stop {
                sequence: stops.len() as i32 + 1,
                pickup_id,
                latitude: lat,
                longitude: lon,
                estimated_volume: volume,
            });
        }
        total_distance_km += haversine_km(current, DEPOT);

        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let route = Route {
            route_id: format!("ROUTE-{id:05}"),
            vehicle_id: data.vehicle_id,
            service_date: data.service_date,
            status: RouteStatus::Optimized.into(),
            stops,
            total_distance_km: (total_distance_km * 100.0).round() / 100.0,
            total_duration_minutes: (total_distance_km / AVG_SPEED_KMPH * 60.0).round(),
        };

        rpc_common::lock(&self.routes).insert(route.route_id.clone(), route.clone());

        tracing::info!(
            route_id = %route.route_id,
            stops = route.stops.len(),
            distance_km = route.total_distance_km,
            caller = %caller.user_id,
            "route optimized"
        );
        Ok(Response::new(route))
    }

    async fn get_route(
        &self,
        request: Request<GetRouteRequest>,
    ) -> Result<Response<Route>, Status> {
        Caller::from_request(&request)?.require_action("route.get")?;
        let id = request.into_inner().route_id;
        validation::identifier("route_id", &id)?;

        let route = rpc_common::lock(&self.routes)
            .get(&id)
            .cloned()
            .ok_or_else(|| rpc_common::not_found("Route", &id))?;
        Ok(Response::new(route))
    }
}
