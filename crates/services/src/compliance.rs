//! Compliance RPC: collection statistics and monthly reports.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use rpc_common::authentication::Caller;
use rpc_common::validation;
use smarttrash_proto::compliance::compliance_service_server::ComplianceService;
use smarttrash_proto::compliance::{
    CollectionStats, ComplianceReport, GenerateMonthlyReportRequest, GetCollectionStatsRequest,
};
use tonic::{Request, Response, Status};

/// Seeded statistics, keyed by `(zone, month)`.
pub struct ComplianceRpc {
    stats: HashMap<(String, String), CollectionStats>,
}

/// Current month as `yyyy-mm` (UTC).
fn current_month() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let days = (secs / 86_400) as i64;

    // civil_from_days (Howard Hinnant).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };

    format!("{year:04}-{month:02}")
}

fn seed_stats(zone: &str, month: &str) -> CollectionStats {
    let (total, completed, volume, diversion, vehicles) = match zone {
        "Kinondoni" => (1_284, 1_147, 2_310.5, 38.5, 14),
        "Ilala" => (1_012, 903, 1_874.0, 34.2, 11),
        "Temeke" => (876, 741, 1_522.7, 31.8, 9),
        _ => (0, 0, 0.0, 0.0, 0),
    };

    CollectionStats {
        zone: zone.to_string(),
        month: month.to_string(),
        total_pickups: total,
        completed_pickups: completed,
        missed_pickups: total - completed,
        total_volume_cubic_metres: volume,
        diversion_rate_percent: diversion,
        active_vehicles: vehicles,
    }
}

impl Default for ComplianceRpc {
    fn default() -> Self {
        let month = current_month();
        let stats = ["Kinondoni", "Ilala", "Temeke"]
            .into_iter()
            .map(|zone| ((zone.to_string(), month.clone()), seed_stats(zone, &month)))
            .collect();

        Self { stats }
    }
}

#[tonic::async_trait]
impl ComplianceService for ComplianceRpc {
    async fn get_collection_stats(
        &self,
        request: Request<GetCollectionStatsRequest>,
    ) -> Result<Response<CollectionStats>, Status> {
        Caller::from_request(&request)?.require_action("dashboard.view")?;
        let data = request.into_inner();

        let zone = if data.zone.trim().is_empty() {
            "Kinondoni".to_string()
        } else {
            validation::text("zone", &data.zone, 64)?;
            data.zone.trim().to_string()
        };
        let month = if data.month.trim().is_empty() {
            current_month()
        } else {
            validation::month(&data.month)?;
            data.month
        };

        let stats = self
            .stats
            .get(&(zone.clone(), month.clone()))
            .cloned()
            .unwrap_or_else(|| seed_stats(&zone, &month));

        Ok(Response::new(stats))
    }

    async fn generate_monthly_report(
        &self,
        request: Request<GenerateMonthlyReportRequest>,
    ) -> Result<Response<ComplianceReport>, Status> {
        let caller = Caller::from_request(&request)?;
        caller.require_action("compliance.report")?;
        let data = request.into_inner();

        let zone = if data.zone.trim().is_empty() {
            "Kinondoni".to_string()
        } else {
            validation::text("zone", &data.zone, 64)?;
            data.zone.trim().to_string()
        };
        let month = if data.month.trim().is_empty() {
            current_month()
        } else {
            validation::month(&data.month)?;
            data.month
        };

        let stats = self
            .stats
            .get(&(zone.clone(), month.clone()))
            .cloned()
            .unwrap_or_else(|| seed_stats(&zone, &month));

        let report = ComplianceReport {
            report_id: format!("RPT-{zone}-{month}").replace(' ', "-"),
            zone,
            month,
            stats: Some(stats),
            document_url: "https://storage.example.com/reports/pending.pdf".into(),
            generated_at_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or_default(),
        };

        tracing::info!(
            report_id = %report.report_id,
            caller = %caller.user_id,
            "compliance report generated"
        );
        Ok(Response::new(report))
    }
}
