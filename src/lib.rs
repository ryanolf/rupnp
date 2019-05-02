#![feature(async_await, await_macro)]

pub mod discovery;

pub mod datatypes;
pub mod device;
pub mod error;
pub mod scpd;
pub mod service;

pub use device::Device;
pub use error::Error;
pub use scpd::SCPD;
pub use service::Service;

mod shared;

pub use ssdp;
