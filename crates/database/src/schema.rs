//! Table definitions (what columns exist).
//!
//! Hand-written to match `migrations/0001_init/up.sql`. If you change the
//! migration, change this file too (or regenerate it with
//! `diesel print-schema` — see `diesel.toml`).

diesel::table! {
    /// The collection requests table.
    pickups (pickup_id) {
        pickup_id -> Text,
        customer_id -> Text,
        waste_type -> Text,
        latitude -> Double,
        longitude -> Double,
        estimated_volume -> Double,
        collected_volume -> Double,
        evidence_url -> Text,
        status -> Text,
        vehicle_id -> Text,
        driver_id -> Text,
        created_at_unix -> BigInt,
        updated_at_unix -> BigInt,
        h3_cell -> Text,
        ward -> Text,
    }
}
