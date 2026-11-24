//! Geo RPC: Tanzania regions and Dar es Salaam wards.

use rpc_common::authentication::Caller;
use rpc_common::validation;
use smarttrash_proto::geo::geo_service_server::GeoService;
use smarttrash_proto::geo::{
    ListRegionsRequest, ListRegionsResponse, ListWardsRequest, ListWardsResponse, Region, Ward,
};
use tonic::{Request, Response, Status};

use crate::users::h3_cell;

const DSM: &str = "DSM";

/// All 31 regions, with their approximate centre.
fn regions() -> Vec<Region> {
    let rows = [
        ("ARU", "Arusha", -3.37, 36.68),
        ("DSM", "Dar es Salaam", -6.79, 39.21),
        ("DOD", "Dodoma", -6.16, 35.75),
        ("GEI", "Geita", -2.87, 32.23),
        ("IRI", "Iringa", -7.77, 35.69),
        ("KAG", "Kagera", -1.90, 31.87),
        ("KAT", "Katavi", -6.37, 31.04),
        ("KIG", "Kigoma", -4.88, 29.63),
        ("KIL", "Kilimanjaro", -3.55, 37.55),
        ("LIN", "Lindi", -9.99, 39.70),
        ("MAN", "Manyara", -4.31, 36.68),
        ("MAR", "Mara", -1.78, 34.00),
        ("MBE", "Mbeya", -8.91, 33.46),
        ("MOR", "Morogoro", -6.82, 37.66),
        ("MTW", "Mtwara", -10.27, 40.18),
        ("MWA", "Mwanza", -2.52, 32.90),
        ("NJO", "Njombe", -9.34, 34.77),
        ("PNB", "Pemba North", -5.03, 39.78),
        ("PNS", "Pemba South", -5.30, 39.72),
        ("PWA", "Pwani", -7.32, 38.83),
        ("RUK", "Rukwa", -8.02, 31.45),
        ("RUV", "Ruvuma", -10.69, 35.65),
        ("SHI", "Shinyanga", -3.66, 33.42),
        ("SIM", "Simiyu", -2.83, 34.15),
        ("SIN", "Singida", -6.20, 34.72),
        ("SON", "Songwe", -8.60, 33.00),
        ("TAB", "Tabora", -5.02, 32.80),
        ("TAN", "Tanga", -5.07, 39.10),
        ("UNB", "Unguja North", -5.94, 39.33),
        ("UNS", "Unguja South", -6.28, 39.55),
        ("UNW", "Unguja Urban/West", -6.16, 39.19),
    ];

    rows.into_iter()
        .map(|(code, name, lat, lng)| Region {
            code: code.to_string(),
            name: name.to_string(),
            center_latitude: lat,
            center_longitude: lng,
            has_wards: code == DSM,
        })
        .collect()
}

/// Only the five Dar es Salaam districts have ward entries.
fn wards() -> Vec<Ward> {
    let rows = [
        ("DSM-KIN", "Kinondoni", -6.78, 39.23),
        ("DSM-ILA", "Ilala", -6.82, 39.28),
        ("DSM-TEM", "Temeke", -6.90, 39.28),
        ("DSM-UBU", "Ubungo", -6.77, 39.18),
        ("DSM-KIG", "Kigamboni", -6.85, 39.35),
    ];

    rows.into_iter()
        .map(|(code, name, lat, lng)| Ward {
            code: code.to_string(),
            name: name.to_string(),
            region_code: DSM.to_string(),
            center_latitude: lat,
            center_longitude: lng,
            h3_cell: h3_cell(lat, lng),
        })
        .collect()
}

pub struct GeoRpc;

#[tonic::async_trait]
impl GeoService for GeoRpc {
    async fn list_regions(
        &self,
        request: Request<ListRegionsRequest>,
    ) -> Result<Response<ListRegionsResponse>, Status> {
        Caller::from_request(&request)?.require_action("geo.view")?;
        Ok(Response::new(ListRegionsResponse {
            regions: regions(),
        }))
    }

    async fn list_wards(
        &self,
        request: Request<ListWardsRequest>,
    ) -> Result<Response<ListWardsResponse>, Status> {
        Caller::from_request(&request)?.require_action("geo.view")?;

        let region = request.into_inner().region_code;
        validation::text("region_code", &region, 8)?;

        let wards = if region == DSM {
            wards()
        } else {
            Vec::new()
        };

        Ok(Response::new(ListWardsResponse { wards }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_dar_has_wards() {
        assert!(regions().iter().filter(|r| r.has_wards).count() == 1);
        assert_eq!(wards().len(), 5);
    }

    #[test]
    fn every_region_has_a_centre_inside_tanzania() {
        for region in regions() {
            assert!((-12.0..=-0.5).contains(&region.center_latitude), "{}", region.code);
            assert!((29.0..=41.0).contains(&region.center_longitude), "{}", region.code);
        }
    }
}
