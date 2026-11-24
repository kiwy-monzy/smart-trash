//! `smarttrash-server`: every RPC service in one process on one port.
//!
//! Served three ways on the same port:
//! * native gRPC (HTTP/2) for service-to-service calls,
//! * gRPC-Web (HTTP/1.1) for the Tauri app's webview,
//! * reflection, so tools can list the API without the `.proto` files.
//!
//! Everything except `AuthService.Login` and reflection needs a bearer token.

use std::net::SocketAddr;
use std::time::Duration;

use http::header::{HeaderName, AUTHORIZATION, CONTENT_TYPE};
use http::Method;
use tonic::service::interceptor::InterceptedService;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};

use smarttrash_proto::auth::auth_service_server::AuthServiceServer;
use smarttrash_proto::compliance::compliance_service_server::ComplianceServiceServer;
use smarttrash_proto::fleet::fleet_service_server::FleetServiceServer;
use smarttrash_proto::geo::geo_service_server::GeoServiceServer;
use smarttrash_proto::identity::identity_service_server::IdentityServiceServer;
use smarttrash_proto::notification::notification_service_server::NotificationServiceServer;
use smarttrash_proto::payment::payment_service_server::PaymentServiceServer;
use smarttrash_proto::pickup::pickup_service_server::PickupServiceServer;
use smarttrash_proto::routing::routing_service_server::RoutingServiceServer;
use smarttrash_proto::scheduling::scheduling_service_server::SchedulingServiceServer;
use smarttrash_proto::tracking::tracking_service_server::TrackingServiceServer;

use smarttrash_services::{
    auth, auth_service, compliance, fleet, geo, identity, notifications, payment, pickup, routing,
    scheduling, tracking, users::UserDirectory,
};

/// Largest message accepted from a client (1 MiB).
const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

fn listen_address() -> Result<SocketAddr, std::net::AddrParseError> {
    std::env::var("RPC_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:50051".to_string())
        .parse()
}

/// Reflection is on unless `GRPC_REFLECTION=false`.
fn reflection_enabled() -> bool {
    !matches!(
        std::env::var("GRPC_REFLECTION").unwrap_or_default().trim(),
        "0" | "false" | "FALSE" | "False"
    )
}

fn token_ttl() -> i64 {
    std::env::var("AUTH_TOKEN_TTL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|ttl| *ttl > 0)
        .unwrap_or(rpc_common::auth_token::DEFAULT_TTL_SECONDS)
        .min(rpc_common::auth_token::MAX_TTL_SECONDS)
}

/// The webview is a different origin from the RPC port.
fn cors_layer() -> CorsLayer {
    let origins = [
        "http://localhost:5173",
        "http://127.0.0.1:5173",
        "http://tauri.localhost",
        "https://tauri.localhost",
        "tauri://localhost",
    ]
    .iter()
    .filter_map(|origin| origin.parse().ok())
    .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([
            CONTENT_TYPE,
            AUTHORIZATION,
            HeaderName::from_static("x-grpc-web"),
            HeaderName::from_static("x-user-agent"),
        ])
        .expose_headers([
            HeaderName::from_static("grpc-status"),
            HeaderName::from_static("grpc-message"),
        ])
        .max_age(Duration::from_secs(600))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rpc_common::settings::load_dotenv();
    rpc_common::telemetry::init();

    let secret = match rpc_common::auth_token::secret_from_env() {
        Ok(secret) => secret,
        Err(err) => {
            eprintln!("configuration error: {err}");
            eprintln!("Set AUTH_TOKEN_SECRET in .env (at least 32 characters).");
            std::process::exit(1);
        }
    };

    let login_secret = secret.clone();
    let users = UserDirectory::default();
    let notifications = std::sync::Arc::new(notifications::NotificationStore::default());
    let auth = move |request: tonic::Request<()>| auth::authenticate(&secret, request);

    let mut connection = database::establish_connection();
    database::run_migrations(&mut connection);

    let address = listen_address()?;
    println!("Smart Trash RPC listening on http://{address} (gRPC + gRPC-Web)");
    if reflection_enabled() {
        println!("gRPC reflection enabled (no token needed for listing)");
    }

    let mut router = tonic::transport::Server::builder()
        .accept_http1(true)
        .tcp_keepalive(Some(Duration::from_secs(60)))
        .http2_keepalive_interval(Some(Duration::from_secs(120)))
        .http2_keepalive_timeout(Some(Duration::from_secs(20)))
        .layer(cors_layer())
        .layer(GrpcWebLayer::new())
        // Sign-in: the only business service without a token.
        .add_service(AuthServiceServer::new(auth_service::AuthRpc::new(
            login_secret,
            token_ttl(),
            users.clone(),
        )))
        .add_service(InterceptedService::new(
            PickupServiceServer::new(pickup::PickupRpc::new(connection))
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            auth.clone(),
        ));

    // Reflection stays outside the auth layer; it only describes the API.
    if reflection_enabled() {
        router = router
            .add_service(
                tonic_reflection::server::Builder::configure()
                    .register_encoded_file_descriptor_set(smarttrash_proto::FILE_DESCRIPTOR_SET)
                    .build_v1()?,
            )
            .add_service(
                tonic_reflection::server::Builder::configure()
                    .register_encoded_file_descriptor_set(smarttrash_proto::FILE_DESCRIPTOR_SET)
                    .build_v1alpha()?,
            );
    }

    router
        .add_service(InterceptedService::new(
            RoutingServiceServer::new(routing::RoutingRpc::default())
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            auth.clone(),
        ))
        .add_service(InterceptedService::new(
            FleetServiceServer::new(fleet::FleetRpc::default())
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            auth.clone(),
        ))
        .add_service(InterceptedService::new(
            TrackingServiceServer::new(tracking::TrackingRpc::new(
            users.clone(),
            notifications.clone(),
        ))
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            auth.clone(),
        ))
        .add_service(InterceptedService::new(
            SchedulingServiceServer::new(scheduling::SchedulingRpc::default())
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            auth.clone(),
        ))
        .add_service(InterceptedService::new(
            IdentityServiceServer::new(identity::IdentityRpc::new(users.clone()))
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            auth.clone(),
        ))
        .add_service(InterceptedService::new(
            PaymentServiceServer::new(payment::PaymentRpc::default())
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            auth.clone(),
        ))
        .add_service(InterceptedService::new(
            GeoServiceServer::new(geo::GeoRpc)
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            auth.clone(),
        ))
        .add_service(InterceptedService::new(
            NotificationServiceServer::new(notifications::NotificationRpc::new(
                notifications.clone(),
            ))
            .max_decoding_message_size(MAX_MESSAGE_BYTES)
            .max_encoding_message_size(MAX_MESSAGE_BYTES),
            auth.clone(),
        ))
        .add_service(InterceptedService::new(
            ComplianceServiceServer::new(compliance::ComplianceRpc::default())
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            auth,
        ))
        .serve_with_shutdown(address, shutdown_signal())
        .await?;

    println!("Smart Trash RPC stopped");
    Ok(())
}

/// Ctrl+C: stop accepting, let running calls finish.
async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %err, "failed to listen for Ctrl+C");
    }
    tracing::info!("shutdown signal received, draining connections");
}
