//! Rust bindings for the BSEC library.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(missing_docs)]
#![allow(clippy::unreadable_literal)]

include!(concat!(env!("OUT_DIR"), "/bsec_bindings.rs"));

// Extra implementation logic for the generated structures.

impl bsec_sensor_configuration_t {
    /// Create new instance of `bsec_sensor_configuration_t`
    ///
    /// # Returns
    /// A new instance of `bsec_sensor_configuration_t` where all
    ///  elements are zeroed out.
    pub fn new() -> Self {
        Self {
            sample_rate: 0.0,
            sensor_id: 0,
        }
    }
}

impl bsec_output_t {
    /// Create new instance of `bsec_output_t`
    ///
    /// # Returns
    /// A new instance of `bsec_output_t` where all elements are
    /// zeroed out.
    pub fn new() -> Self {
        Self {
            accuracy: 0,
            sensor_id: 0,
            signal: 0.0,
            signal_dimensions: 0,
            time_stamp: 0,
        }
    }
}
