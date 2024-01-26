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
    #[must_use]
    pub fn new() -> Self {
        Self {
            sample_rate: 0.0,
            sensor_id: 0,
        }
    }
}

impl Default for bsec_sensor_configuration_t {
    fn default() -> Self {
        Self::new()
    }
}

impl bsec_output_t {
    /// Create new instance of `bsec_output_t`
    ///
    /// # Returns
    /// A new instance of `bsec_output_t` where all elements are
    /// zeroed out.
    #[must_use]
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

impl Default for bsec_output_t {
    fn default() -> Self {
        Self::new()
    }
}

impl bsec_bme_settings_t {
    /// Create a new instance of `bsec_bme_settings_t`
    ///
    /// # Returns
    /// A new instance of `bsec_bme_settings_t` where all elements
    /// are zeroed out.
    pub fn new() -> Self {
        Self {
            heater_duration: 0,
            heater_duration_profile: [0; 10],
            heater_profile_len: 0,
            heater_temperature: 0,
            heater_temperature_profile: [0; 10],
            humidity_oversampling: 0,
            next_call: 0,
            op_mode: 0,
            pressure_oversampling: 0,
            process_data: 0,
            run_gas: 0,
            temperature_oversampling: 0,
            trigger_measurement: 0,
        }
    }
}

impl Default for bsec_bme_settings_t {
    fn default() -> Self {
        Self::new()
    }
}
