//! # `smoltcp` implementation for `rmodbus`
//!
//! For using Modbus in embedded or `no_std` environments.
//!
//! ## Features
//!
//! - `defmt-03`: Adds `defmt::Format` derives for all types and enables the equivalent feature for all dependencies that support it.

#![no_std]

pub mod server;

/// Default Modbus TCP port.
pub const PORT: u16 = 502;
