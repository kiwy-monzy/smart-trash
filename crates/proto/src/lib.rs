//! Generated protobuf types and service contracts for Smart Trash.
//!
//! Every `.proto` file in the workspace root `proto/` directory is compiled
//! here once, so services, clients and the API gateway all share a single
//! strongly typed contract.
//!
//! `include_proto!` takes the proto *package* name (which is also the
//! generated file name), e.g. `package smarttrash.pickup;` →
//! `include_proto!("smarttrash.pickup")`.

/// `smarttrash.identity`: users, roles and authorization.
pub mod identity {
    tonic::include_proto!("smarttrash.identity");
}

/// `smarttrash.auth`: sign-in over gRPC.
pub mod auth {
    tonic::include_proto!("smarttrash.auth");
}

/// `smarttrash.pickup`: collection requests and their lifecycle.
pub mod pickup {
    tonic::include_proto!("smarttrash.pickup");
}

/// `smarttrash.scheduling`: collection windows and slot assignment.
pub mod scheduling {
    tonic::include_proto!("smarttrash.scheduling");
}

/// `smarttrash.routing`: route optimization.
pub mod routing {
    tonic::include_proto!("smarttrash.routing");
}

/// `smarttrash.fleet`: trucks and drivers.
pub mod fleet {
    tonic::include_proto!("smarttrash.fleet");
}

/// `smarttrash.tracking`: live GPS positions.
pub mod tracking {
    tonic::include_proto!("smarttrash.tracking");
}

/// `smarttrash.payment`: invoices and payments.
pub mod payment {
    tonic::include_proto!("smarttrash.payment");
}

/// `smarttrash.compliance`: council statistics and reports.
pub mod compliance {
    tonic::include_proto!("smarttrash.compliance");
}

/// `smarttrash.notification`: 'truck nearby' alerts.
pub mod notification {
    tonic::include_proto!("smarttrash.notification");
}

/// `smarttrash.geo`: Tanzania regions and Dar es Salaam wards.
pub mod geo {
    tonic::include_proto!("smarttrash.geo");
}

/// Compiled descriptor set, served by gRPC reflection.
pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/smarttrash_descriptor.bin"));
