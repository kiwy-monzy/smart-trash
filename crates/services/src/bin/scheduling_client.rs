//! Demo client: another service calling Pickup RPC with its own token.
//!
//! Start `smarttrash-server` first, then run this binary.

use smarttrash_proto::pickup::pickup_service_client::PickupServiceClient;
use smarttrash_proto::pickup::{
    CreatePickupRequest, GetPickupRequest, ListPickupsRequest, WasteType,
};
use tonic::metadata::MetadataValue;
use tonic::Request;

fn authorized<T>(message: T, token: &str) -> Result<Request<T>, Box<dyn std::error::Error>> {
    let mut request = Request::new(message);
    let value: MetadataValue<tonic::metadata::Ascii> = format!("Bearer {token}").parse()?;
    request.metadata_mut().insert("authorization", value);
    Ok(request)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rpc_common::settings::load_dotenv();
    rpc_common::telemetry::init();

    let endpoint = std::env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:50051".into());

    // Act as HOUSEHOLD-0421, like the gateway would.
    let secret = rpc_common::auth_token::secret_from_env()?;
    let token = rpc_common::auth_token::issue(
        &secret,
        "HOUSEHOLD-0421",
        rpc_common::authentication::Role::Household,
        300,
    )?;

    let mut client = PickupServiceClient::connect(endpoint).await?;

    let created = client
        .create_pickup(authorized(
            CreatePickupRequest {
                customer_id: "HOUSEHOLD-0421".into(),
                waste_type: WasteType::Recyclable.into(),
                latitude: -6.7565,
                longitude: 39.2049,
                estimated_volume: 2.5,
            },
            &token,
        )?)
        .await?
        .into_inner();

    println!(
        "created: {} status={:?}",
        created.pickup_id,
        created.status()
    );

    let fetched = client
        .get_pickup(authorized(
            GetPickupRequest {
                pickup_id: created.pickup_id.clone(),
            },
            &token,
        )?)
        .await?
        .into_inner();

    println!(
        "fetched: {} customer={} status={:?}",
        fetched.pickup_id,
        fetched.customer_id,
        fetched.status()
    );

    let listed = client
        .list_pickups(authorized(
            ListPickupsRequest {
                page_size: 10,
                status_filter: 0,
            },
            &token,
        )?)
        .await?
        .into_inner();

    println!("pickups visible to HOUSEHOLD-0421: {}", listed.pickups.len());

    Ok(())
}
