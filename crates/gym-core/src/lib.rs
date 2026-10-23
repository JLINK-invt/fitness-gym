//! Pure domain rules for Fitness Gym.
//!
//! This crate performs no I/O and must stay independent from Tauri and SQLite
//! so the business rules can be tested in isolation.

pub mod access;
pub mod auth;
pub mod error;
pub mod events;
pub mod ids;
pub mod rbac;

pub use error::CoreError;
pub use ids::{OrganizationId, UserId};

/// Version reported by the core module in health checks.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Stable identifier of the local core service.
pub const SERVICE_NAME: &str = "fitness-gym";
