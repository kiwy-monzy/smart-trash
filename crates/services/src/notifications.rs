//! In-memory "truck nearby" notifications.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rpc_common::authentication::Caller;
use rpc_common::validation;
use smarttrash_proto::notification::notification_service_server::NotificationService;
use smarttrash_proto::notification::{
    ListNotificationsRequest, ListNotificationsResponse, Notification,
};
use tonic::{Request, Response, Status};

use crate::users::UserDirectory;

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

#[derive(Default)]
pub struct NotificationStore {
    items: Mutex<Vec<Notification>>,
    next: AtomicI64,
}

impl NotificationStore {
    /// Newest first, only the caller's own alerts.
    pub fn list_for(&self, user_id: &str, limit: usize) -> Vec<Notification> {
        let items = rpc_common::lock(&self.items);
        let mut mine: Vec<Notification> = items
            .iter()
            .filter(|n| n.user_id == user_id)
            .cloned()
            .collect();
        mine.sort_by(|a, b| b.created_at_unix.cmp(&a.created_at_unix));
        mine.truncate(limit);
        mine
    }

    /// Alert every user whose H3 cell touches the vehicle's cell.
    pub fn notify_nearby(&self, users: &UserDirectory, vehicle_id: &str, cell: &str) {
        let Ok(origin) = cell.parse::<h3o::CellIndex>() else {
            return;
        };
        let neighbours = origin.grid_disk::<Vec<_>>(1);

        for user in users.list() {
            if user.h3_cell.is_empty() {
                continue;
            }
            let Ok(home) = user.h3_cell.parse::<h3o::CellIndex>() else {
                continue;
            };
            if !neighbours.contains(&home) {
                continue;
            }

            let id = self.next.fetch_add(1, Ordering::Relaxed) + 1;
            let notification = Notification {
                notification_id: format!("NTF-{id:05}"),
                user_id: user.user_id.clone(),
                title: "Truck nearby".to_string(),
                body: format!("{vehicle_id} is collecting in your area"),
                h3_cell: user.h3_cell.clone(),
                created_at_unix: unix_now(),
            };

            let mut items = rpc_common::lock(&self.items);
            // Keep the demo list short and avoid repeating the same alert.
            if items
                .iter()
                .any(|n| n.user_id == notification.user_id && n.body == notification.body)
            {
                continue;
            }
            items.push(notification);
            if items.len() > 500 {
                let drop = items.len() - 500;
                items.drain(0..drop);
            }
        }
    }
}

pub struct NotificationRpc {
    store: std::sync::Arc<NotificationStore>,
}

impl NotificationRpc {
    pub fn new(store: std::sync::Arc<NotificationStore>) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl NotificationService for NotificationRpc {
    async fn list_notifications(
        &self,
        request: Request<ListNotificationsRequest>,
    ) -> Result<Response<ListNotificationsResponse>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("notification.list")?;

        let limit = validation::page_size(request.into_inner().limit, 20, 50) as usize;
        let notifications = self.store.list_for(&caller.user_id, limit);

        Ok(Response::new(ListNotificationsResponse { notifications }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vehicle_only_alerts_users_in_the_same_or_next_cell() {
        let users = UserDirectory::default();
        let store = NotificationStore::default();

        let household = users.get("HOUSEHOLD-0421").expect("seed user");
        store.notify_nearby(&users, "VEH-002", &household.h3_cell);

        let mine = store.list_for("HOUSEHOLD-0421", 10);
        assert_eq!(mine.len(), 1);
        assert!(mine[0].body.contains("VEH-002"));
        assert!(mine[0].body.contains("nearby") || mine[0].body.contains("collecting"));

        // Somebody far away is not notified.
        assert!(store.list_for("BUSINESS-12", 10).is_empty());
    }
}
