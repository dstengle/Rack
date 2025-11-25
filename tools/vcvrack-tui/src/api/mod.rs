//! API module for VCV Rack HTTP communication

pub mod client;
pub mod types;

pub use client::RackApiClient;
pub use types::*;
