//! Scheduling RPC: collection windows (zone + time slot) with a capacity limit.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;

use rpc_common::authentication::Caller;
use rpc_common::validation;
use smarttrash_proto::scheduling::scheduling_service_server::SchedulingService;
use smarttrash_proto::scheduling::{
    CollectionWindow, CreateWindowRequest, GetWindowRequest, ListWindowsRequest,
    ListWindowsResponse, SchedulePickupRequest, WindowStatus,
};
use tonic::{Request, Response, Status};

const MAX_WINDOW_SECONDS: i64 = 72 * 60 * 60;
const MAX_CAPACITY: i32 = 10_000;

pub struct SchedulingRpc {
    windows: Mutex<HashMap<String, CollectionWindow>>,
    next_id: AtomicI64,
}

impl Default for SchedulingRpc {
    fn default() -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
            next_id: AtomicI64::new(0),
        }
    }
}

#[tonic::async_trait]
impl SchedulingService for SchedulingRpc {
    async fn create_window(
        &self,
        request: Request<CreateWindowRequest>,
    ) -> Result<Response<CollectionWindow>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("window.schedule")?;
        let data = request.into_inner();

        validation::text("zone", &data.zone, 64)?;
        if data.end_unix <= data.start_unix {
            return Err(rpc_common::invalid_argument(
                "end_unix must be after start_unix",
            ));
        }
        if data.end_unix - data.start_unix > MAX_WINDOW_SECONDS {
            return Err(rpc_common::invalid_argument(
                "a collection window may not be longer than 72 hours",
            ));
        }
        if data.capacity_limit < 0 || data.capacity_limit > MAX_CAPACITY {
            return Err(rpc_common::invalid_argument(format!(
                "capacity_limit must be within [0, {MAX_CAPACITY}]"
            )));
        }

        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let window = CollectionWindow {
            window_id: format!("WIN-{id:05}"),
            zone: data.zone,
            start_unix: data.start_unix,
            end_unix: data.end_unix,
            status: WindowStatus::Open.into(),
            pickup_ids: Vec::new(),
            capacity_limit: data.capacity_limit,
        };

        rpc_common::lock(&self.windows).insert(window.window_id.clone(), window.clone());

        tracing::info!(
            window_id = %window.window_id,
            zone = %window.zone,
            caller = %caller.user_id,
            "window created"
        );
        Ok(Response::new(window))
    }

    async fn schedule_pickup(
        &self,
        request: Request<SchedulePickupRequest>,
    ) -> Result<Response<CollectionWindow>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("window.schedule")?;
        let data = request.into_inner();

        validation::identifier("window_id", &data.window_id)?;
        validation::identifier("pickup_id", &data.pickup_id)?;

        let mut windows = rpc_common::lock(&self.windows);
        let window = windows
            .get_mut(&data.window_id)
            .ok_or_else(|| rpc_common::not_found("Window", &data.window_id))?;

        if window.status() != WindowStatus::Open {
            return Err(rpc_common::failed_precondition(format!(
                "window '{}' is {:?}, expected OPEN",
                data.window_id,
                window.status()
            )));
        }
        if window.pickup_ids.contains(&data.pickup_id) {
            return Err(rpc_common::failed_precondition(format!(
                "pickup '{}' already scheduled in window '{}'",
                data.pickup_id, data.window_id
            )));
        }
        if window.capacity_limit > 0 && window.pickup_ids.len() as i32 >= window.capacity_limit {
            return Err(rpc_common::failed_precondition(format!(
                "window '{}' is at capacity ({})",
                data.window_id, window.capacity_limit
            )));
        }

        window.pickup_ids.push(data.pickup_id);
        tracing::debug!(
            window_id = %data.window_id,
            caller = %caller.user_id,
            "pickup scheduled"
        );
        Ok(Response::new(window.clone()))
    }

    async fn get_window(
        &self,
        request: Request<GetWindowRequest>,
    ) -> Result<Response<CollectionWindow>, Status> {
        Caller::from_request(&request)?.require_action("window.get")?;
        let id = request.into_inner().window_id;
        validation::identifier("window_id", &id)?;

        let window = rpc_common::lock(&self.windows)
            .get(&id)
            .cloned()
            .ok_or_else(|| rpc_common::not_found("Window", &id))?;
        Ok(Response::new(window))
    }

    async fn list_windows(
        &self,
        request: Request<ListWindowsRequest>,
    ) -> Result<Response<ListWindowsResponse>, Status> {
        Caller::from_request(&request)?.require_action("window.get")?;

        let mut windows: Vec<CollectionWindow> =
            rpc_common::lock(&self.windows).values().cloned().collect();
        windows.sort_by(|a, b| a.start_unix.cmp(&b.start_unix));

        Ok(Response::new(ListWindowsResponse { windows }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rpc_common::authentication::Role;
    use tonic::Code;

    fn council_request<T>(message: T) -> Request<T> {
        let mut request = Request::new(message);
        request
            .extensions_mut()
            .insert(Caller::new("COUNCIL-01", Role::Council));
        request
    }

    #[tokio::test]
    async fn a_window_cannot_end_before_it_starts() {
        let service = SchedulingRpc::default();
        let request = council_request(CreateWindowRequest {
            zone: "Kinondoni".into(),
            start_unix: 1_000,
            end_unix: 900,
            capacity_limit: 10,
        });

        let status = service.create_window(request).await.expect_err("must fail");
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn a_call_without_identity_is_unauthenticated() {
        let service = SchedulingRpc::default();
        let request = Request::new(CreateWindowRequest {
            zone: "Kinondoni".into(),
            start_unix: 1_000,
            end_unix: 2_000,
            capacity_limit: 10,
        });

        let status = service.create_window(request).await.expect_err("must fail");
        assert_eq!(status.code(), Code::Unauthenticated);
    }
}
