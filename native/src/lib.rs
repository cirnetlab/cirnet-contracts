pub mod capture;
pub mod generator;
pub mod validation;
pub mod wire_json;

#[cfg(feature = "python-tests")]
mod python_tests;

pub mod hardware_v2 {
    include!(concat!(env!("OUT_DIR"), "/cirnet.hardware.v2.rs"));

    pub const CANONICAL_SERVICE: &str = "cirnet.hardware.v2.HardwareService";
    pub const LEGACY_SERVICE: &str = "netcircuit.hardware.v2.HardwareService";
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/hardware-v2.pb"));
    pub const LEGACY_FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/hardware-v2-legacy.pb"));
}
