//! Data and types for interconnect between tasks.
/// Structure for holding data from all of the sensors
use crate::bsec::StructuredOutputs;
use crate::veml7700::VemlOutput;

/// Structure used to hold data collected by the sensor hub.
#[derive(Clone, Copy)]
pub struct SensorHubData {
    /// Data from the BME688 sensor
    pub bsec: StructuredOutputs,

    /// Data from the VEML7700 sensor
    pub veml: VemlOutput,
}

impl SensorHubData {
    /// Create a new instance of the structure
    pub fn new() -> Self {
        Self {
            bsec: StructuredOutputs::new(),
            veml: VemlOutput::new(),
        }
    }
}
