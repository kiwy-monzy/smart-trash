//! Shared helpers for the RPC services.

pub mod auth_token;
pub mod authentication;
pub mod demo;
pub mod errors;
pub mod settings;
pub mod telemetry;
pub mod validation;

pub use errors::{failed_precondition, invalid_argument, not_found, unauthenticated, unavailable};

use std::sync::{Mutex, MutexGuard};

/// Lock a mutex, recovering the data if another task panicked while holding it.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
