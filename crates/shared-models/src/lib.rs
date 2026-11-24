//! JSON models shared by the API gateway and the apps.
//!
//! The wire format of the HTTP API is defined here (snake_case, lowercase
//! enum tags), and conversion to/from the protobuf contract lives next to it,
//! so adding a field means touching one crate.

use serde::{Deserialize, Serialize};
use smarttrash_proto::compliance::CollectionStats;
use smarttrash_proto::pickup::{
    CreatePickupRequest, Pickup, PickupStatus as ProtoPickupStatus, WasteType as ProtoWasteType,
};
use smarttrash_proto::tracking::ReportLocationRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickupStatus {
    Pending,
    Scheduled,
    Assigned,
    InProgress,
    Completed,
    Cancelled,
}

impl From<ProtoPickupStatus> for PickupStatus {
    fn from(status: ProtoPickupStatus) -> Self {
        match status {
            ProtoPickupStatus::Pending => Self::Pending,
            ProtoPickupStatus::Scheduled => Self::Scheduled,
            ProtoPickupStatus::Assigned => Self::Assigned,
            ProtoPickupStatus::InProgress => Self::InProgress,
            ProtoPickupStatus::Completed => Self::Completed,
            ProtoPickupStatus::Cancelled => Self::Cancelled,
            ProtoPickupStatus::Unspecified => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WasteType {
    Organic,
    Recyclable,
    General,
    Hazardous,
    Construction,
}

impl From<WasteType> for ProtoWasteType {
    fn from(waste_type: WasteType) -> Self {
        match waste_type {
            WasteType::Organic => Self::Organic,
            WasteType::Recyclable => Self::Recyclable,
            WasteType::General => Self::General,
            WasteType::Hazardous => Self::Hazardous,
            WasteType::Construction => Self::Construction,
        }
    }
}

/// Body of `POST /api/pickups`.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePickupDto {
    pub customer_id: String,
    pub waste_type: WasteType,
    pub latitude: f64,
    pub longitude: f64,
    pub estimated_volume: f64,
}

impl CreatePickupDto {
    /// Translate the HTTP body into the gRPC request.
    pub fn into_request(self) -> CreatePickupRequest {
        CreatePickupRequest {
            customer_id: self.customer_id,
            waste_type: ProtoWasteType::from(self.waste_type) as i32,
            latitude: self.latitude,
            longitude: self.longitude,
            estimated_volume: self.estimated_volume,
        }
    }
}

/// A pickup as returned by the HTTP API.
#[derive(Debug, Clone, Serialize)]
pub struct PickupDto {
    pub pickup_id: String,
    pub status: PickupStatus,
    pub customer_id: String,
    pub waste_type: WasteType,
    pub latitude: f64,
    pub longitude: f64,
    pub estimated_volume: f64,
    pub collected_volume: f64,
    pub vehicle_id: String,
    pub driver_id: String,
    pub evidence_url: String,
    pub created_at_unix: i64,
    pub updated_at_unix: i64,
    pub h3_cell: String,
    pub ward: String,
}

impl From<Pickup> for PickupDto {
    fn from(pickup: Pickup) -> Self {
        // Read the accessors before the Strings are moved out.
        let status = pickup.status().into();
        let waste_type = match pickup.waste_type() {
            ProtoWasteType::Organic => WasteType::Organic,
            ProtoWasteType::Recyclable => WasteType::Recyclable,
            ProtoWasteType::General => WasteType::General,
            ProtoWasteType::Hazardous => WasteType::Hazardous,
            ProtoWasteType::Construction => WasteType::Construction,
            ProtoWasteType::Unspecified => WasteType::General,
        };

        Self {
            pickup_id: pickup.pickup_id,
            status,
            customer_id: pickup.customer_id,
            waste_type,
            latitude: pickup.latitude,
            longitude: pickup.longitude,
            estimated_volume: pickup.estimated_volume,
            collected_volume: pickup.collected_volume,
            vehicle_id: pickup.vehicle_id,
            driver_id: pickup.driver_id,
            evidence_url: pickup.evidence_url,
            created_at_unix: pickup.created_at_unix,
            updated_at_unix: pickup.updated_at_unix,
            h3_cell: pickup.h3_cell,
            ward: pickup.ward,
        }
    }
}

/// Body of `POST /api/tracking/locations` (driver app GPS ping).
#[derive(Debug, Clone, Deserialize)]
pub struct ReportLocationDto {
    pub vehicle_id: String,
    pub driver_id: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub speed_mps: f64,
    #[serde(default)]
    pub heading_degrees: f64,
}

impl ReportLocationDto {
    /// Translate the HTTP body into the gRPC request. `recorded_at_unix = 0`
    /// makes the tracking service stamp the time it received the ping.
    pub fn into_request(self) -> ReportLocationRequest {
        ReportLocationRequest {
            vehicle_id: self.vehicle_id,
            driver_id: self.driver_id,
            latitude: self.latitude,
            longitude: self.longitude,
            speed_mps: self.speed_mps,
            heading_degrees: self.heading_degrees,
            recorded_at_unix: 0,
        }
    }
}

/// Council statistics as returned by `GET /api/stats`.
#[derive(Debug, Clone, Serialize)]
pub struct CollectionStatsDto {
    pub zone: String,
    pub month: String,
    pub total_pickups: i32,
    pub completed_pickups: i32,
    pub missed_pickups: i32,
    pub total_volume_cubic_metres: f64,
    pub diversion_rate_percent: f64,
    pub active_vehicles: i32,
}

impl From<CollectionStats> for CollectionStatsDto {
    fn from(stats: CollectionStats) -> Self {
        Self {
            zone: stats.zone,
            month: stats.month,
            total_pickups: stats.total_pickups,
            completed_pickups: stats.completed_pickups,
            missed_pickups: stats.missed_pickups,
            total_volume_cubic_metres: stats.total_volume_cubic_metres,
            diversion_rate_percent: stats.diversion_rate_percent,
            active_vehicles: stats.active_vehicles,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_dto_serialises_snake_case_enum() {
        let json = r#"{"customer_id":"R1","waste_type":"recyclable","latitude":-6.8,"longitude":39.2,"estimated_volume":2.5}"#;
        let dto: CreatePickupDto = serde_json::from_str(json).expect("valid dto");
        assert_eq!(dto.waste_type, WasteType::Recyclable);
        assert_eq!(dto.into_request().waste_type, ProtoWasteType::Recyclable as i32);
    }

    #[test]
    fn unknown_waste_type_is_rejected() {
        let json = r#"{"customer_id":"R1","waste_type":"nuclear","latitude":0.0,"longitude":0.0,"estimated_volume":1.0}"#;
        assert!(serde_json::from_str::<CreatePickupDto>(json).is_err());
    }
}
