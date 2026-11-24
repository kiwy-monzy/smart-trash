//! HTTP API gateway: REST/JSON for the clients, gRPC for the services.
//!
//! Every `/api/…` route except `POST /api/auth/login` needs a bearer token.
//! The same token is forwarded to the RPC layer, which checks it again.

mod auth;
mod ratelimit;

use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::{DefaultBodyLimit, Extension, Path as UrlPath, Query, State};
use axum::http::{header, HeaderName, HeaderValue, Method, StatusCode};
use axum::middleware;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use tonic::metadata::{Ascii, MetadataValue};
use tonic::transport::{Channel, Endpoint};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;

use rpc_common::auth_token;
use rpc_common::authentication::{Caller, Role};
use shared_models::{CollectionStatsDto, CreatePickupDto, PickupDto, ReportLocationDto};
use smarttrash_proto::compliance::compliance_service_client::ComplianceServiceClient;
use smarttrash_proto::compliance::GetCollectionStatsRequest;
use smarttrash_proto::pickup::pickup_service_client::PickupServiceClient;
use smarttrash_proto::pickup::{
    AssignPickupRequest, GetPickupRequest, ListPickupsRequest, PickupStatus,
};
use smarttrash_proto::tracking::tracking_service_client::TrackingServiceClient;

use crate::auth::{AuthConfig, Session};
use crate::ratelimit::RateLimiter;

const MAX_BODY_BYTES: usize = 64 * 1024;
const MAX_PAGE_SIZE: i32 = 200;
const RPC_TIMEOUT: Duration = Duration::from_secs(15);

type RpcError = (StatusCode, String);

#[derive(Clone)]
struct AppState {
    pickups: PickupServiceClient<Channel>,
    tracking: TrackingServiceClient<Channel>,
    compliance: ComplianceServiceClient<Channel>,
    auth: AuthConfig,
    limiter: RateLimiter,
}

/// gRPC status → HTTP status.
fn rpc_status(err: tonic::Status) -> RpcError {
    let code = match err.code() {
        tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
        tonic::Code::NotFound => StatusCode::NOT_FOUND,
        tonic::Code::AlreadyExists => StatusCode::CONFLICT,
        tonic::Code::FailedPrecondition => StatusCode::CONFLICT,
        tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        tonic::Code::PermissionDenied => StatusCode::FORBIDDEN,
        tonic::Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
        tonic::Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, err.message().to_string())
}

/// Channel that connects on first use, so the gateway starts without the RPC server.
fn lazy_channel(url: String) -> Result<Channel, tonic::transport::Error> {
    Ok(Endpoint::from_shared(url)?.connect_lazy())
}

/// Wrap a message in a gRPC request that carries the caller's token.
fn rpc_request<T>(
    state: &AppState,
    session: &Session,
    message: T,
) -> Result<tonic::Request<T>, RpcError> {
    let mut request = tonic::Request::new(message);
    let value: MetadataValue<Ascii> = state
        .auth
        .forward_value(&session.token)
        .parse()
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "could not attach credentials".to_string(),
            )
        })?;
    request.metadata_mut().insert("authorization", value);
    Ok(request)
}

fn authorize(caller: &Caller, action: &str) -> Result<(), RpcError> {
    caller.require_action(action).map_err(rpc_status)
}

fn forbidden(message: impl Into<String>) -> RpcError {
    (StatusCode::FORBIDDEN, message.into())
}

async fn create_pickup(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(dto): Json<CreatePickupDto>,
) -> Result<(StatusCode, Json<PickupDto>), RpcError> {
    authorize(&session.caller, "pickup.create")?;

    // Only a council may book for somebody else.
    if session.caller.role != Role::Council && dto.customer_id != session.caller.user_id {
        return Err(forbidden(
            "customer_id must match the signed-in user; only councils may book for others",
        ));
    }

    let request = rpc_request(&state, &session, dto.into_request())?;
    let mut pickups = state.pickups.clone();
    let response = pickups.create_pickup(request).await.map_err(rpc_status)?;

    Ok((StatusCode::CREATED, Json(response.into_inner().into())))
}

#[derive(Deserialize, Default)]
struct ListQuery {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    page_size: Option<i32>,
}

fn parse_status(raw: &str) -> Option<PickupStatus> {
    match raw.to_ascii_lowercase().as_str() {
        "pending" => Some(PickupStatus::Pending),
        "scheduled" => Some(PickupStatus::Scheduled),
        "assigned" => Some(PickupStatus::Assigned),
        "in_progress" => Some(PickupStatus::InProgress),
        "completed" => Some(PickupStatus::Completed),
        "cancelled" => Some(PickupStatus::Cancelled),
        _ => None,
    }
}

async fn list_pickups(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<PickupDto>>, RpcError> {
    authorize(&session.caller, "pickup.list")?;

    let status_filter = match query.status.as_deref().filter(|s| !s.is_empty()) {
        None => PickupStatus::Unspecified as i32,
        Some(raw) => {
            parse_status(raw)
                .ok_or((StatusCode::BAD_REQUEST, format!("unknown status '{raw}'")))?
                as i32
        }
    };
    let page_size = query.page_size.unwrap_or(0).clamp(0, MAX_PAGE_SIZE);

    let request = rpc_request(
        &state,
        &session,
        ListPickupsRequest {
            page_size,
            status_filter,
        },
    )?;
    let mut pickups = state.pickups.clone();
    let response = pickups.list_pickups(request).await.map_err(rpc_status)?;

    Ok(Json(
        response
            .into_inner()
            .pickups
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

async fn get_pickup(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    UrlPath(pickup_id): UrlPath<String>,
) -> Result<Json<PickupDto>, RpcError> {
    authorize(&session.caller, "pickup.get")?;

    let request = rpc_request(&state, &session, GetPickupRequest { pickup_id })?;
    let mut pickups = state.pickups.clone();
    let response = pickups.get_pickup(request).await.map_err(rpc_status)?;

    Ok(Json(response.into_inner().into()))
}

#[derive(Deserialize)]
struct AssignPickupDto {
    vehicle_id: String,
    driver_id: String,
}

async fn assign_pickup(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    UrlPath(pickup_id): UrlPath<String>,
    Json(dto): Json<AssignPickupDto>,
) -> Result<Json<PickupDto>, RpcError> {
    authorize(&session.caller, "pickup.assign")?;

    let request = rpc_request(
        &state,
        &session,
        AssignPickupRequest {
            pickup_id,
            vehicle_id: dto.vehicle_id,
            driver_id: dto.driver_id,
        },
    )?;
    let mut pickups = state.pickups.clone();
    let response = pickups.assign_pickup(request).await.map_err(rpc_status)?;

    Ok(Json(response.into_inner().into()))
}

async fn report_location(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(dto): Json<ReportLocationDto>,
) -> Result<(StatusCode, Json<serde_json::Value>), RpcError> {
    authorize(&session.caller, "tracking.report")?;

    // A driver reports only for itself.
    if dto.driver_id != session.caller.user_id {
        return Err(forbidden(
            "driver_id must match the signed-in driver; crews cannot report for each other",
        ));
    }

    let request = rpc_request(&state, &session, dto.into_request())?;
    let mut tracking = state.tracking.clone();
    let response = tracking.report_location(request).await.map_err(rpc_status)?;

    let ack = response.into_inner();
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "vehicle_id": ack.vehicle_id,
            "accepted_at_unix": ack.accepted_at_unix,
        })),
    ))
}

#[derive(Deserialize, Default)]
struct StatsQuery {
    #[serde(default)]
    zone: Option<String>,
    #[serde(default)]
    month: Option<String>,
}

async fn collection_stats(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<CollectionStatsDto>, RpcError> {
    authorize(&session.caller, "dashboard.view")?;

    let request = rpc_request(
        &state,
        &session,
        GetCollectionStatsRequest {
            zone: query.zone.unwrap_or_default(),
            month: query.month.unwrap_or_default(),
        },
    )?;
    let mut compliance = state.compliance.clone();
    let response = compliance
        .get_collection_stats(request)
        .await
        .map_err(rpc_status)?;

    Ok(Json(response.into_inner().into()))
}

#[derive(Deserialize)]
struct LoginDto {
    user_id: String,
}

/// Demo sign-in: only active when `AUTH_DEMO_LOGIN=true`.
async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<serde_json::Value>, RpcError> {
    if !state.auth.demo_login {
        return Err(forbidden(
            "demo sign-in is disabled; configure a real credential provider",
        ));
    }
    if !auth_token::valid_user_id(&dto.user_id) {
        return Err((StatusCode::BAD_REQUEST, "user_id is not valid".to_string()));
    }

    let Some(role) = auth::demo_role(&dto.user_id) else {
        return Err((StatusCode::UNAUTHORIZED, "unknown user".to_string()));
    };

    let (token, expires_at_unix) = state.auth.issue(&dto.user_id, role).map_err(|err| {
        tracing::error!(error = %err, "could not issue a session token");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "could not issue a session".to_string(),
        )
    })?;

    tracing::info!(user_id = %dto.user_id, "session issued");
    Ok(Json(json!({
        "token": token,
        "user_id": dto.user_id,
        "role": auth_token::role_name(role),
        "expires_at_unix": expires_at_unix,
    })))
}

async fn me(Extension(session): Extension<Session>) -> Json<serde_json::Value> {
    Json(json!({
        "user_id": session.caller.user_id,
        "role": auth_token::role_name(session.caller.role),
        "expires_at_unix": session.expires_at_unix,
    }))
}

async fn health() -> &'static str {
    "ok"
}

/// Small index so a browser hitting the port sees something useful.
async fn index() -> Json<serde_json::Value> {
    Json(json!({
        "service": "smart-trash-gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "routes": [
            "POST /api/auth/login",
            "GET  /api/auth/me",
            "GET|POST /api/pickups",
            "GET  /api/pickups/{id}",
            "POST /api/pickups/{id}/assign",
            "POST /api/tracking/locations",
            "GET  /api/stats",
            "GET  /health",
        ],
        "client": "apps/app (Tauri, gRPC-Web on :50051)",
    }))
}

/// The Tauri app and the Vite dev server are separate origins from the API.
fn cors_layer() -> CorsLayer {
    let origins = [
        "http://localhost:5173",
        "http://127.0.0.1:5173",
        "http://tauri.localhost",
        "https://tauri.localhost",
        "tauri://localhost",
    ]
    .iter()
    .filter_map(|origin| origin.parse::<HeaderValue>().ok())
    .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(Duration::from_secs(600))
}

fn router(state: AppState) -> Router {
    let auth_config = state.auth.clone();
    let limiter = state.limiter.clone();

    Router::new()
        .route("/health", get(health))
        .route("/", get(index))
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", get(me))
        .route("/api/pickups", post(create_pickup).get(list_pickups))
        .route("/api/pickups/{pickup_id}", get(get_pickup))
        .route("/api/pickups/{pickup_id}/assign", post(assign_pickup))
        .route("/api/tracking/locations", post(report_location))
        .route("/api/stats", get(collection_stats))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(middleware::from_fn_with_state(
            auth_config,
            auth::require_bearer,
        ))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            RPC_TIMEOUT,
        ))
        .layer(CatchPanicLayer::new())
        .layer(middleware::from_fn_with_state(limiter, ratelimit::limit))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'unsafe-inline'; \
                 style-src 'self' 'unsafe-inline'; img-src 'self' data:; \
                 connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; \
                 form-action 'self'",
            ),
        ))
        // Outermost: answers the CORS preflight before any auth check runs.
        .layer(cors_layer())
        .with_state(state)
}

fn rate_limit_from_env() -> u32 {
    std::env::var("RATE_LIMIT_PER_MINUTE")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(600)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rpc_common::settings::load_dotenv();
    rpc_common::telemetry::init();

    let auth_config = match AuthConfig::from_env() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("configuration error: {err}");
            eprintln!("Set AUTH_TOKEN_SECRET in .env (at least 32 characters).");
            std::process::exit(1);
        }
    };
    if auth_config.demo_login {
        tracing::warn!(
            "AUTH_DEMO_LOGIN is on: /api/auth/login issues tokens for the seeded demo users. \
             Turn it off and plug in a real credential check before deploying."
        );
    }

    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:50051".into());

    let state = AppState {
        pickups: PickupServiceClient::new(lazy_channel(rpc_url.clone())?),
        tracking: TrackingServiceClient::new(lazy_channel(rpc_url.clone())?),
        compliance: ComplianceServiceClient::new(lazy_channel(rpc_url)?),
        auth: auth_config,
        limiter: RateLimiter::new(rate_limit_from_env()),
    };

    let address: SocketAddr = std::env::var("GATEWAY_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;

    println!("API gateway listening on http://{address}");

    axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    println!("API gateway stopped");
    Ok(())
}

async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %err, "failed to listen for Ctrl+C");
    }
    tracing::info!("shutdown signal received, draining connections");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_codes_map_to_the_expected_http_status() {
        assert_eq!(
            rpc_status(tonic::Status::invalid_argument("x")).0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            rpc_status(tonic::Status::permission_denied("x")).0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            rpc_status(tonic::Status::unauthenticated("x")).0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            rpc_status(tonic::Status::resource_exhausted("x")).0,
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn status_filters_are_case_insensitive() {
        assert_eq!(parse_status("PENDING"), Some(PickupStatus::Pending));
        assert_eq!(parse_status("in_progress"), Some(PickupStatus::InProgress));
        assert_eq!(parse_status("unknown"), None);
    }

    #[test]
    fn the_index_lists_the_routes() {
        assert!(env!("CARGO_PKG_VERSION").starts_with(char::is_numeric));
    }
}
