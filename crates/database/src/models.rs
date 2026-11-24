//! Rust structs that map to the tables (rows in, rows out).

use diesel::prelude::*;

use crate::schema::pickups;

/// One row of `pickups`, as read from the database.
/// The field order must match `schema::pickups`.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = pickups)]
pub struct PickupRow {
    pub pickup_id: String,
    pub customer_id: String,
    pub waste_type: String,
    pub latitude: f64,
    pub longitude: f64,
    pub estimated_volume: f64,
    pub collected_volume: f64,
    pub evidence_url: String,
    pub status: String,
    pub vehicle_id: String,
    pub driver_id: String,
    pub created_at_unix: i64,
    pub updated_at_unix: i64,
    /// H3 cell of the pickup location (resolution 9).
    pub h3_cell: String,
    /// Ward the pickup belongs to.
    pub ward: String,
}

/// A new pickup to insert. Columns with database defaults (id, collected
/// volume, vehicle, …) are left out on purpose.
#[derive(Debug, Insertable)]
#[diesel(table_name = pickups)]
pub struct NewPickup {
    pub customer_id: String,
    pub waste_type: String,
    pub latitude: f64,
    pub longitude: f64,
    pub estimated_volume: f64,
    pub status: String,
    pub created_at_unix: i64,
    pub updated_at_unix: i64,
    pub h3_cell: String,
    pub ward: String,
}
