//! Main BSEC logic
// pub mod bindings;
// FIXME: Make private and have rust-based wrappers around everything?
#[allow(clippy::module_name_repetitions)]
pub mod bsec_bindings;

use std::num::TryFromIntError;

// Allow here because this woiud
use self::bsec_bindings::{
    bsec_bme_settings_t, bsec_do_steps, bsec_get_version, bsec_init, bsec_input_t,
    bsec_library_return_t, bsec_output_t, bsec_sensor_configuration_t, bsec_sensor_control,
    bsec_update_subscription, bsec_version_t, BSEC_E_CONFIG_CRCMISMATCH, BSEC_E_CONFIG_EMPTY,
    BSEC_E_CONFIG_FAIL, BSEC_E_CONFIG_FEATUREMISMATCH, BSEC_E_CONFIG_INSUFFICIENTBUFFER,
    BSEC_E_CONFIG_INSUFFICIENTWORKBUFFER, BSEC_E_CONFIG_INVALIDSTRINGSIZE,
    BSEC_E_CONFIG_VERSIONMISMATCH, BSEC_E_DOSTEPS_DUPLICATEINPUT, BSEC_E_DOSTEPS_INVALIDINPUT,
    BSEC_E_DOSTEPS_VALUELIMITS, BSEC_E_PARSE_SECTIONEXCEEDSWORKBUFFER,
    BSEC_E_SET_INVALIDCHANNELIDENTIFIER, BSEC_E_SET_INVALIDLENGTH, BSEC_E_SU_DUPLICATEGATE,
    BSEC_E_SU_GATECOUNTEXCEEDSARRAY, BSEC_E_SU_HIGHHEATERONDURATION, BSEC_E_SU_INVALIDSAMPLERATE,
    BSEC_E_SU_MULTGASSAMPLINTVL, BSEC_E_SU_SAMPLERATELIMITS, BSEC_E_SU_SAMPLINTVLINTEGERMULT,
    BSEC_E_SU_WRONGDATARATE, BSEC_INPUT_GASRESISTOR, BSEC_INPUT_HEATSOURCE, BSEC_INPUT_HUMIDITY,
    BSEC_INPUT_PRESSURE, BSEC_INPUT_PROFILE_PART, BSEC_INPUT_TEMPERATURE,
    BSEC_I_DOSTEPS_NOOUTPUTSRETURNABLE, BSEC_I_SU_GASESTIMATEPRECEDENCE,
    BSEC_I_SU_SUBSCRIBEDOUTPUTGATES, BSEC_MAX_PHYSICAL_SENSOR, BSEC_NUMBER_OUTPUTS, BSEC_OK,
    BSEC_OUTPUT_BREATH_VOC_EQUIVALENT, BSEC_OUTPUT_CO2_EQUIVALENT, BSEC_OUTPUT_GAS_ESTIMATE_1,
    BSEC_OUTPUT_GAS_ESTIMATE_2, BSEC_OUTPUT_GAS_ESTIMATE_3, BSEC_OUTPUT_GAS_ESTIMATE_4,
    BSEC_OUTPUT_GAS_PERCENTAGE, BSEC_OUTPUT_IAQ, BSEC_OUTPUT_RAW_GAS, BSEC_OUTPUT_RAW_GAS_INDEX,
    BSEC_OUTPUT_RAW_HUMIDITY, BSEC_OUTPUT_RAW_PRESSURE, BSEC_OUTPUT_RAW_TEMPERATURE,
    BSEC_OUTPUT_RUN_IN_STATUS, BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_HUMIDITY,
    BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_TEMPERATURE, BSEC_OUTPUT_STABILIZATION_STATUS,
    BSEC_OUTPUT_STATIC_IAQ, BSEC_W_DOSTEPS_EXCESSOUTPUTS, BSEC_W_DOSTEPS_GASINDEXMISS,
    BSEC_W_DOSTEPS_TSINTRADIFFOUTOFRANGE, BSEC_W_SC_CALL_TIMING_VIOLATION,
    BSEC_W_SC_MODEXCEEDULPTIMELIMIT, BSEC_W_SC_MODINSUFFICIENTWAITTIME, BSEC_W_SU_MODINNOULP,
    BSEC_W_SU_UNKNOWNOUTPUTGATE,
};

use embedded_hal::i2c::I2c;
use esp_idf_hal::delay::FreeRtos;

use crate::bme68x::{
    BME68xAddr, BME68xData, BME68xDev, BME68xError, BME68xIntf, BME68xOpMode, BME68xOs,
};

/// Rust-Native wrapper for the BSEC Error codes.
/// Has a few additional error codes beyond what the BSEC library provides.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy)]
pub enum BsecError {
    /// Success
    Ok,

    /// Invalid input to `bsec_do_steps`
    DoStepsInvalidInput,

    /// Value passed to `bsec_do_steps` is out of range
    DoStepsValueLimits,

    /// Timestamp passed to `bsec_do_steps` is smaller than the previous
    DoStepsTsIntRadifOutOfRange,

    /// Same Input provided more than once
    DoStepsDuplicateInput,

    /// No memory allocated for returning outputs
    DoStepsNoOutputsReturnable,

    /// Not enough memory to hold return values
    DoStepsExcessOutputs,

    /// Gas index not provided
    DoStepsGasIndexMiss,

    /// Data rate of requested output is 0
    WrongDataRate,

    /// Sample rate not supported for given output
    SampleRateLimits,

    /// Duplicate ouput requested
    DuplicateGate,

    /// Invalid sampling rate
    InvalidSampleRate,

    /// Not Enough memopry to hold physical sensor data
    GateCountExceedsArray,

    /// Invalid output sample interval
    SampleIntervalIntegerMult,

    /// Invalid sampel output interval when gas sensing required
    MultGasSampleInterval,

    /// Measurement duration longer than requested sample interval
    HighHeaterDuration,

    /// Output sensor ID not in the valid range.
    UnknownOutputGate,

    /// ULP+ cannot be requrested in non-ulp mode
    ModInNoULP,

    /// No virtual sensor outputs were requested.
    SubscribedOutputGates,

    /// Gas Estimate is subscribed and takes precendence.
    GasEstimatePrecedence,

    /// Work buffer size not sufficent
    SectionExceedsWorkBuffer,

    /// Configuration failed
    ConfigFail,

    /// Serialized settings are for a different BSEC version
    ConfigVersionMisMatch,

    /// Serialized Enabled Features are for a different BSEC version
    ConfigFeatureMismatch,

    /// CRC of serialized settings does not match
    ConfigCRCMisMatch,

    /// Serialized Configuration is too small to be valid
    ConfigEmpty,

    /// Provided work buffer not large enough for the desired string
    ConfigInsufficentWorkBuffer,

    /// String size does not match specified string size
    ConfigInvalidStringSize,

    /// String buffer insufficent to hold entire configuration
    ConfigInsufficentBuffer,

    /// Internal warning that size of work buffer in setConfig is incorrect
    SetInvalidChannelIdentifier,

    /// Internal error code
    SetInvalidLength,

    /// Difference between actual and defined sampling rate too large
    CallTimingViolation,

    /// ULP+ not allowed becuase ULP measurement just/about to occur
    ModExceedULPTimeLimit,

    /// ULP+ Not allowed becuase not enough time since last ULP+
    ModInsufficentWaitTime,

    /// Error with the back-end BME68x Driver
    DriverError {
        /// Back End driver error
        error: BME68xError,
    },

    /// Error converting between numeric typoes
    NumericConversionErrror,

    /// Unknown error code
    UnknownError {
        /// The unknown error code
        code: bsec_library_return_t,
    },
}

impl From<bsec_library_return_t> for BsecError {
    #![allow(non_upper_case_globals)]
    fn from(value: bsec_library_return_t) -> Self {
        match value {
            BSEC_OK => Self::Ok,
            BSEC_E_DOSTEPS_INVALIDINPUT => Self::DoStepsInvalidInput,
            BSEC_E_DOSTEPS_VALUELIMITS => Self::DoStepsValueLimits,
            BSEC_W_DOSTEPS_TSINTRADIFFOUTOFRANGE => Self::DoStepsTsIntRadifOutOfRange,
            BSEC_E_DOSTEPS_DUPLICATEINPUT => Self::DoStepsDuplicateInput,
            BSEC_I_DOSTEPS_NOOUTPUTSRETURNABLE => Self::DoStepsNoOutputsReturnable,
            BSEC_W_DOSTEPS_EXCESSOUTPUTS => Self::DoStepsExcessOutputs,
            BSEC_W_DOSTEPS_GASINDEXMISS => Self::DoStepsGasIndexMiss,
            BSEC_E_SU_WRONGDATARATE => Self::WrongDataRate,
            BSEC_E_SU_SAMPLERATELIMITS => Self::SampleRateLimits,
            BSEC_E_SU_DUPLICATEGATE => Self::DuplicateGate,
            BSEC_E_SU_INVALIDSAMPLERATE => Self::InvalidSampleRate,
            BSEC_E_SU_GATECOUNTEXCEEDSARRAY => Self::GateCountExceedsArray,
            BSEC_E_SU_SAMPLINTVLINTEGERMULT => Self::SampleIntervalIntegerMult,
            BSEC_E_SU_MULTGASSAMPLINTVL => Self::MultGasSampleInterval,
            BSEC_E_SU_HIGHHEATERONDURATION => Self::HighHeaterDuration,
            BSEC_W_SU_UNKNOWNOUTPUTGATE => Self::UnknownOutputGate,
            BSEC_W_SU_MODINNOULP => Self::ModInNoULP,
            BSEC_I_SU_SUBSCRIBEDOUTPUTGATES => Self::SubscribedOutputGates,
            BSEC_I_SU_GASESTIMATEPRECEDENCE => Self::GasEstimatePrecedence,
            BSEC_E_PARSE_SECTIONEXCEEDSWORKBUFFER => Self::SectionExceedsWorkBuffer,
            BSEC_E_CONFIG_FAIL => Self::ConfigFail,
            BSEC_E_CONFIG_VERSIONMISMATCH => Self::ConfigVersionMisMatch,
            BSEC_E_CONFIG_FEATUREMISMATCH => Self::ConfigFeatureMismatch,
            BSEC_E_CONFIG_CRCMISMATCH => Self::ConfigCRCMisMatch,
            BSEC_E_CONFIG_EMPTY => Self::ConfigEmpty,
            BSEC_E_CONFIG_INSUFFICIENTWORKBUFFER => Self::ConfigInsufficentWorkBuffer,
            BSEC_E_CONFIG_INVALIDSTRINGSIZE => Self::ConfigInvalidStringSize,
            BSEC_E_CONFIG_INSUFFICIENTBUFFER => Self::ConfigInsufficentBuffer,
            BSEC_E_SET_INVALIDCHANNELIDENTIFIER => Self::SetInvalidChannelIdentifier,
            BSEC_E_SET_INVALIDLENGTH => Self::SetInvalidLength,
            BSEC_W_SC_CALL_TIMING_VIOLATION => Self::CallTimingViolation,
            BSEC_W_SC_MODEXCEEDULPTIMELIMIT => Self::ModExceedULPTimeLimit,
            BSEC_W_SC_MODINSUFFICIENTWAITTIME => Self::ModInsufficentWaitTime,
            _ => Self::UnknownError { code: value },
        }
    }
}

impl From<BME68xError> for BsecError {
    fn from(value: BME68xError) -> Self {
        Self::DriverError { error: value }
    }
}

impl From<TryFromIntError> for BsecError {
    fn from(_value: TryFromIntError) -> Self {
        Self::NumericConversionErrror
    }
}

/// Special version of `bsec_output_t`
///
/// This contains almost the same data as `bsec_output_t`, but with the
/// additional benefit of also having a .valid member to indicate if the given
/// output signal was provided at the most recent periodic processing iteration
/// See the documentation for `bsec_output_t` for further details
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtualSensorData {
    /// Time stamp in ns of the signal generation
    pub time_stamp: i64,

    /// Output value
    pub signal: f32,

    /// Reserved for future use
    pub signal_dimensions: u8,

    /// Accuracy indication of the data
    pub accuracy: u8,

    /// Indicating that the data is valid
    pub valid: bool,
}

impl VirtualSensorData {
    /// Create a new empty instance of the structure
    #[must_use]
    pub fn new() -> Self {
        Self {
            time_stamp: 0,
            signal: 0.0,
            signal_dimensions: 0,
            accuracy: 0,
            valid: false,
        }
    }
}

/// Well-structured collection of BSEC virtual sensors
/// This is used to provide the same information as in the normal
/// output array, except in a more well-formatted form, so finding the correct
/// sensor output does not require searching through the array
/// See `bsec_virtual_sensor_t`  in the BSEC documentation for information about
/// virtual sensors
#[derive(Debug, Clone, Copy, Default)]
pub struct StructuredOutputs {
    /// Indoor air quality
    pub iaq: VirtualSensorData,
    /// Unscaled indoor air quality
    pub static_iaq: VirtualSensorData,
    /// Equivlent CO2 estimate (ppm)
    pub co2_eq: VirtualSensorData,
    /// Breath VOC estimate (ppm)
    pub breath_voc_eq: VirtualSensorData,
    /// Raw temperature (degrees C)
    pub raw_temp: VirtualSensorData,
    /// Raw pressure (Pa)
    pub raw_pressure: VirtualSensorData,
    /// Raw humidity (%)
    pub raw_humidity: VirtualSensorData,
    /// Raw gas sensor (Ohm)
    pub raw_gas: VirtualSensorData,
    /// Stabilization status
    pub stabilization_status: VirtualSensorData,
    /// Sensor Run in status
    pub run_in_status: VirtualSensorData,
    /// Heat Compensated temp (C)
    pub compensated_temp: VirtualSensorData,
    /// Heat Compensated Humidity (C)
    pub compensated_humidity: VirtualSensorData,
    /// Percentage of min/max filter gas (%)
    pub gas_percentage: VirtualSensorData,
    /// Gas channel 1 estimate
    pub gas_estimate_1: VirtualSensorData,
    /// Gas channel 2 estimate
    pub gas_estimate_2: VirtualSensorData,
    /// Gas channel 3 estimate
    pub gas_estimate_3: VirtualSensorData,
    /// Gas channel 4 estimate
    pub gas_estimate_4: VirtualSensorData,
    /// gas heater profile index.
    pub raw_gas_index: VirtualSensorData,
}

impl StructuredOutputs {
    /// Create a new empty instance of the structure
    #[must_use]
    pub fn new() -> Self {
        Self {
            breath_voc_eq: VirtualSensorData::new(),
            co2_eq: VirtualSensorData::new(),
            compensated_humidity: VirtualSensorData::new(),
            compensated_temp: VirtualSensorData::new(),
            gas_estimate_1: VirtualSensorData::new(),
            gas_estimate_2: VirtualSensorData::new(),
            gas_estimate_3: VirtualSensorData::new(),
            gas_estimate_4: VirtualSensorData::new(),
            gas_percentage: VirtualSensorData::new(),
            iaq: VirtualSensorData::new(),
            raw_gas: VirtualSensorData::new(),
            raw_gas_index: VirtualSensorData::new(),
            raw_humidity: VirtualSensorData::new(),
            raw_pressure: VirtualSensorData::new(),
            raw_temp: VirtualSensorData::new(),
            run_in_status: VirtualSensorData::new(),
            stabilization_status: VirtualSensorData::new(),
            static_iaq: VirtualSensorData::new(),
        }
    }
}

/// Main BSEC Implementation structure
pub struct Bsec<I2C> {
    /// The BME68x device to use with the BSEC library
    bme: BME68xDev<I2C>,

    /// Output data from BSEC
    outputs: StructuredOutputs,

    /// Offset to apply to teh temperature measurement to correct for sensor or enclosure bias
    temp_offset: f32,

    /// Most recently read sensor settings
    // TODO: Rust-native structure instead of the C one
    sensor_settings: bsec_bme_settings_t,

    /// Current periodic_processing iteration time (in ns)
    curr_time_ns: i64,
}

// TODO: Rust enum for error code
// TODO: Rust enum for bsec_virtual_sensor_t

impl<I2C: I2c> Bsec<I2C> {
    /// Initialize the device for use with the BSEC system
    /// # Arguments
    /// * `i2c`: The i2c bus to use for communication with the sensor
    /// * `temp_offset`: The offset to apply to the temperature measurement, to correct for sensor or enclosure bias.
    pub fn new(i2c: I2C, temp_offset: f32) -> Self {
        // TODO: Finish initialzers
        Self {
            bme: BME68xDev::new(
                i2c,
                BME68xAddr::HIGH,
                25,
                BME68xIntf::I2CIntf,
                Box::new(FreeRtos::delay_us),
            ),
            outputs: StructuredOutputs::new(),
            temp_offset,
            // FIXME: Can we create a ::new() method for this?
            sensor_settings: bsec_bme_settings_t {
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
            },
            curr_time_ns: 0,
        }
    }

    /// Initialize the BSEC library.
    ///
    /// # Errors
    /// Returns an error if initializing the library failed.
    // TODO: Make this part of new()?
    pub fn init(&mut self) -> Result<(), BsecError> {
        self.bme.init()?;
        to_err(unsafe { bsec_init() })?;
        Ok(())
    }

    /// Get the version of the BSEC library
    ///
    /// # Returns
    /// The version of the BSEC library
    ///
    /// # Errors
    /// Returns an error if reading the version fails.
    // TODO: Rust native version structure
    pub fn get_version(&self) -> Result<bsec_version_t, BsecError> {
        let mut version = bsec_version_t {
            major: 0,
            minor: 0,
            major_bugfix: 0,
            minor_bugfix: 0,
        };
        to_err(unsafe { bsec_get_version(&mut version) })?;
        Ok(version)
    }

    /// Update the requested sensor data
    ///
    /// # Arguments
    /// * `requested_virtual_sensors`: The requested virtual sensors to subscribe to
    ///
    /// # Errors
    /// Returns and error if updating the subscription failed
    pub fn update_subscription(
        &self,
        requested_virtual_sensors: &[bsec_sensor_configuration_t],
    ) -> Result<(), BsecError> {
        // FIXME: See if we can add derive for copy to the struct to make this smaller.
        let mut required_sensor_settings: [bsec_sensor_configuration_t;
            BSEC_MAX_PHYSICAL_SENSOR as usize] = [
            bsec_sensor_configuration_t {
                sample_rate: 0.0,
                sensor_id: 0,
            },
            bsec_sensor_configuration_t {
                sample_rate: 0.0,
                sensor_id: 0,
            },
            bsec_sensor_configuration_t {
                sample_rate: 0.0,
                sensor_id: 0,
            },
            bsec_sensor_configuration_t {
                sample_rate: 0.0,
                sensor_id: 0,
            },
            bsec_sensor_configuration_t {
                sample_rate: 0.0,
                sensor_id: 0,
            },
            bsec_sensor_configuration_t {
                sample_rate: 0.0,
                sensor_id: 0,
            },
            bsec_sensor_configuration_t {
                sample_rate: 0.0,
                sensor_id: 0,
            },
            bsec_sensor_configuration_t {
                sample_rate: 0.0,
                sensor_id: 0,
            },
        ];
        let mut n_required_sensor_settings: u8 = BSEC_MAX_PHYSICAL_SENSOR.try_into()?;
        to_err(unsafe {
            bsec_update_subscription(
                requested_virtual_sensors.as_ptr(),
                requested_virtual_sensors.len().try_into()?,
                required_sensor_settings.as_mut_ptr(),
                &mut n_required_sensor_settings,
            )
        })?;

        Ok(())
    }

    /// Subscribe to all non gas-scan sensors
    ///
    /// # Arguments
    /// * `sample_rate`: The sameple rate to subscribe all sensors to
    ///
    /// # Errors
    /// Returns an error if subscribing fails
    ///
    // TODO: Enum for sample rate
    pub fn subscribe_all_non_scan(&self, sample_rate: f32) -> Result<(), BsecError> {
        let requested_sensors = [
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_RAW_TEMPERATURE.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_RAW_PRESSURE.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_RAW_HUMIDITY.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_RAW_GAS.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_IAQ.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_STATIC_IAQ.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_CO2_EQUIVALENT.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_BREATH_VOC_EQUIVALENT.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_TEMPERATURE.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_HUMIDITY.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_STABILIZATION_STATUS.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_RUN_IN_STATUS.try_into()?,
            },
            bsec_sensor_configuration_t {
                sample_rate,
                sensor_id: BSEC_OUTPUT_GAS_PERCENTAGE.try_into()?,
            },
        ];
        self.update_subscription(&requested_sensors)
    }

    ///  Read data from the sensor and process it
    ///
    /// # Arguments
    /// * `timestamp_ns`: Current system timestamp in microseconds
    ///
    /// # Errors
    /// Errors if reading and processing the data failed.
    ///
    /// # Panics
    /// Panics if an attempt to use the sensor's sequential mode is made.
    // FIXME: Return either library or bme68x error based on error code
    pub fn periodic_process(&mut self, timestamp_ns: i64) -> Result<(), BsecError> {
        let mut sensor_settings = bsec_bme_settings_t {
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
        };

        self.curr_time_ns = timestamp_ns;

        to_err(unsafe { bsec_sensor_control(timestamp_ns, &mut sensor_settings) })?;
        self.sensor_settings = sensor_settings;

        match BME68xOpMode::from(self.sensor_settings.op_mode) {
            BME68xOpMode::ForcedMode => self.configure_sensor_forced(),
            BME68xOpMode::ParallelMode => self.configure_sensor_parallel(),
            BME68xOpMode::SleepMode => self.bme.set_op_mode(BME68xOpMode::SleepMode),
            BME68xOpMode::SequentialMode => panic!("Sequential Op Not Supported"),
        }?;

        if (self.sensor_settings.trigger_measurement != 0)
            && !matches!(
                BME68xOpMode::from(self.sensor_settings.op_mode),
                BME68xOpMode::SleepMode,
            )
        {
            // FIXME: Replace with match
            let result = self
                .bme
                .get_data(BME68xOpMode::from(self.sensor_settings.op_mode));
            if result.is_err() && matches!(result.unwrap_err(), BME68xError::NoNewData) {
            } else if result.is_err() {
                result?;
            } else {
                let (data, n_data) = result?;
                if n_data > 0 {
                    for entry in data.iter().take(n_data as usize) {
                        self.process_data(entry)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the most recent set of output data from the structure
    ///
    /// # Returns
    /// The most recent output data from the structure
    pub fn get_output_data(&self) -> StructuredOutputs {
        self.outputs
    }

    /// Get the timestamp (in ns) for when the next call to `periodic_process` should occur
    ///
    /// # Returns
    /// Timestamp (in ns) to when the next call to `periodic_process`
    /// should occur
    pub fn get_next_call_time(&self) -> i64 {
        self.sensor_settings.next_call
    }

    ///  Get the next call time in microseconds
    ///
    /// # Returns
    /// Time at which `periodic_processing` should be called, in
    /// microseconds
    pub fn get_next_call_time_us(&self) -> i64 {
        self.get_next_call_time() / 1000
    }

    /// Configure the sensor for a forced measurement
    ///
    /// # Errors
    /// Returns an error if configuring the sensor fails
    fn configure_sensor_forced(&mut self) -> Result<(), BME68xError> {
        let mut conf = self.bme.get_config()?;
        conf.os_hum = BME68xOs::from(self.sensor_settings.humidity_oversampling);
        conf.os_temp = BME68xOs::from(self.sensor_settings.temperature_oversampling);
        conf.os_pres = BME68xOs::from(self.sensor_settings.pressure_oversampling);
        self.bme.set_config(&conf)?;
        self.bme.set_heatr_conf_forced(
            self.sensor_settings.heater_temperature,
            self.sensor_settings.heater_duration,
        )?;
        self.bme.set_op_mode(BME68xOpMode::ForcedMode)
    }

    /// Configure the sensor for a parallel measurement
    ///
    /// # Errors
    /// Returns and error if configuring the sensor fails
    fn configure_sensor_parallel(&mut self) -> Result<(), BME68xError> {
        let mut conf = self.bme.get_config()?;
        conf.os_hum = BME68xOs::from(self.sensor_settings.humidity_oversampling);
        conf.os_temp = BME68xOs::from(self.sensor_settings.temperature_oversampling);
        conf.os_pres = BME68xOs::from(self.sensor_settings.pressure_oversampling);
        self.bme.set_config(&conf)?;
        self.bme.set_heatr_conf_parallel(
            &self.sensor_settings.heater_temperature_profile,
            &self.sensor_settings.heater_duration_profile,
        )?;
        self.bme.set_op_mode(BME68xOpMode::ParallelMode)
    }

    /// Process the data and update internal record of most recent data
    ///
    /// Arguments
    /// * `data`: The data from the sensor to process
    ///
    /// # Errors
    /// Returns an error if processing the data failed.
    fn process_data(&mut self, data: &BME68xData) -> Result<(), BsecError> {
        let mut inputs: Vec<bsec_input_t> = Vec::new();
        // Conditionalyl add sensor data
        self.add_sig_cond(BSEC_INPUT_PRESSURE, data.pressure, &mut inputs);
        self.add_sig_cond(BSEC_INPUT_HUMIDITY, data.humidity, &mut inputs);
        self.add_sig_cond(BSEC_INPUT_TEMPERATURE, data.temperature, &mut inputs);
        self.add_sig_cond(BSEC_INPUT_GASRESISTOR, data.gas_resistance, &mut inputs);
        self.add_sig_cond(BSEC_INPUT_HEATSOURCE, self.temp_offset, &mut inputs);

        // TODO: BSEC_INPUT_DISABLE_BASELINE_TRACKER

        // TODO: Not 100% sure what this is. Need to check datasheet
        self.add_sig_cond(
            BSEC_INPUT_PROFILE_PART,
            if self.sensor_settings.op_mode == BME68xOpMode::ForcedMode.into() {
                0.0
            } else {
                f32::from(data.gas_index)
            },
            &mut inputs,
        );

        if !inputs.is_empty() {
            // FIXME: Clone/copy impl for the structure?
            let mut outputs: [bsec_output_t; BSEC_NUMBER_OUTPUTS as usize] = [
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
                bsec_output_t {
                    accuracy: 0,
                    sensor_id: 0,
                    signal: 0.0,
                    signal_dimensions: 0,
                    time_stamp: 0,
                },
            ];
            let mut num_outputs: u8 = outputs.len().try_into()?;
            to_err(unsafe {
                bsec_do_steps(
                    inputs.as_ptr(),
                    inputs.len().try_into()?,
                    outputs.as_mut_ptr(),
                    &mut num_outputs,
                )
            })?;
            self.update_output_structure(&mut outputs, usize::from(num_outputs));
        }

        Ok(())
    }

    /// Conditionally aed a value to the inputs array used for updating a subscription
    ///
    /// # Arguments
    /// * `input_signal`: The signal type to add conditionally
    /// * `value`: The value to add
    /// * `n_inputs`: Current number of inputs
    /// * `inputs`: The input array
    ///
    /// # Returns
    /// The new number of inputs
    fn add_sig_cond(&self, input_signal: u32, value: f32, inputs: &mut Vec<bsec_input_t>) {
        if check_input_request(self.sensor_settings.process_data, input_signal) {
            inputs.push(bsec_input_t {
                sensor_id: input_signal.try_into().unwrap(),
                signal: value,
                time_stamp: self.curr_time_ns,
                signal_dimensions: 0,
            });
        }
    }
    /// Update the outputs structure with newly read sensor data
    ///
    /// # Arguments
    /// * `outputs`: The array of output dat from the BSEC library
    /// * `num_outputs`: The number of outputs in the output array
    ///
    /// # Panics
    /// Will panic if the requested data type is not known.
    fn update_output_structure(&mut self, outputs: &mut [bsec_output_t], num_outputs: usize) {
        for output in outputs.iter().take(num_outputs) {
            let data: &mut VirtualSensorData = match u32::from(output.sensor_id) {
                BSEC_OUTPUT_IAQ => &mut self.outputs.iaq,
                BSEC_OUTPUT_STATIC_IAQ => &mut self.outputs.static_iaq,
                BSEC_OUTPUT_CO2_EQUIVALENT => &mut self.outputs.co2_eq,
                BSEC_OUTPUT_BREATH_VOC_EQUIVALENT => &mut self.outputs.breath_voc_eq,
                BSEC_OUTPUT_RAW_TEMPERATURE => &mut self.outputs.raw_temp,
                BSEC_OUTPUT_RAW_PRESSURE => &mut self.outputs.raw_pressure,
                BSEC_OUTPUT_RAW_HUMIDITY => &mut self.outputs.raw_humidity,
                BSEC_OUTPUT_RAW_GAS => &mut self.outputs.raw_gas,
                BSEC_OUTPUT_STABILIZATION_STATUS => &mut self.outputs.stabilization_status,
                BSEC_OUTPUT_RUN_IN_STATUS => &mut self.outputs.run_in_status,
                BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_TEMPERATURE => {
                    &mut self.outputs.compensated_temp
                }
                BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_HUMIDITY => {
                    &mut self.outputs.compensated_humidity
                }
                BSEC_OUTPUT_GAS_PERCENTAGE => &mut self.outputs.gas_percentage,
                BSEC_OUTPUT_GAS_ESTIMATE_1 => &mut self.outputs.gas_estimate_1,
                BSEC_OUTPUT_GAS_ESTIMATE_2 => &mut self.outputs.gas_estimate_2,
                BSEC_OUTPUT_GAS_ESTIMATE_3 => &mut self.outputs.gas_estimate_3,
                BSEC_OUTPUT_GAS_ESTIMATE_4 => &mut self.outputs.gas_estimate_4,
                BSEC_OUTPUT_RAW_GAS_INDEX => &mut self.outputs.raw_gas_index,
                _ => panic!("Unknown sensor output type"),
            };

            data.valid = true;
            data.accuracy = output.accuracy;
            data.signal = output.signal;
            data.signal_dimensions = output.signal_dimensions;
            data.time_stamp = output.time_stamp;
        }
    }
}

/// Check if the given signal is requested
///
/// # Arguments
/// * `value`: The value to check
/// * `shift`: The shift to check for
///
/// # Returns
/// Whether or not the siganl wass requested
#[inline(always)]
fn check_input_request(value: u32, shift: u32) -> bool {
    (value) & (1 << ((shift) - 1)) != 0
}

/// Wrap a BSEC library return to a result structure
///
/// # Arguments
/// * `result`: The result to wrap
///
/// # Errors
/// Returns the error code wrapped in result
#[inline(always)]
fn to_err(result: bsec_library_return_t) -> Result<(), bsec_library_return_t> {
    if result != 0 {
        Err(result)
    } else {
        Ok(())
    }
}
