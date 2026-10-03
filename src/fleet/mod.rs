//! Fleet version governance subsystem.
//! Enables declarative version pinning and driving native harness package managers.

pub mod driver;
pub mod drivers;
pub mod status;

pub use driver::{
    FleetAction, FleetHarnessDriver, FleetHarnessStatus, FleetStatusReport, HarnessAlignment,
};
pub use drivers::all_drivers;
pub use status::generate_fleet_status_report;
