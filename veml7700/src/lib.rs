//! Logic for accessing a VEML7700 sensor attached over I2C
use embedded_hal::i2c::I2c;

/// I2C Address of the sensor
const VEML_ADDR: u8 = 0x10;

/// Base scale for the sensor (at min gain and integration time)
const ALS_BASE_SCALE: f32 = 0.0036;

/// Enumeration of the VEML7700's registers
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum VemlRegister {
    Config = 0x00,
    ALSHighThreshold = 0x01,
    ALSLowThreshold = 0x02,
    PowerSaving = 0x03,
    ALSValue = 0x04,
    WhiteValue = 0x05,
    ALSInterruptStatus = 0x06,
}

/// Enumeration of the power saving modes of the sensor
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum VemlPowerSavingMode {
    /// Mode 1
    Mode1 = 0b00,

    /// Mode 2
    Mode2 = 0b01,

    /// Mode 3
    Mode3 = 0b10,

    /// Mode 4
    Mode4 = 0b11,
}

impl From<u16> for VemlPowerSavingMode {
    fn from(value: u16) -> Self {
        match value {
            0b00 => Self::Mode1,
            0b01 => Self::Mode2,
            0b10 => Self::Mode3,
            0b11 => Self::Mode4,
            _ => panic!("Can not convert {value} into VemlPowerSavingMode"),
        }
    }
}

/// Enumeration of the possible gain values for the sensor
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum VemlGain {
    /// 1X gain
    Gain1 = 0x00,

    /// 2X Gain
    Gain2 = 0x01,

    /// 1/8 Gain
    Gain1_8 = 0x02,

    /// 1/4 Gain
    Gain1_4 = 0x04,
}
impl From<u16> for VemlGain {
    /// Try to create enum from integer
    ///
    /// # Arguments
    /// * `value`: The value to create the enum from
    ///
    /// # Returns
    /// Result of trying to create the enum.
    ///
    /// # Panics
    /// Will panic if attempting to convert an unsupported value to the enum
    fn from(value: u16) -> Self {
        match value {
            0b00 => VemlGain::Gain1,
            0b01 => VemlGain::Gain2,
            0b10 => VemlGain::Gain1_8,
            0b11 => VemlGain::Gain1_4,
            _ => panic!("Can not convert {value} into VemlGain"),
        }
    }
}

/// Enumeration of integration times for the sensor.
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum VemlIntegration {
    /// 25ms integration time
    Int25 = 0b1100,

    /// 50ms integration time
    Int50 = 0b1000,

    /// 100ms integration time
    Int100 = 0b0000,

    /// 200ms integration time
    Int200 = 0b0001,

    /// 400ms integration time
    Int400 = 0b0010,

    /// 800ms integration time
    Int800 = 0b0011,
}

impl From<u16> for VemlIntegration {
    /// Try to create enum from integer
    ///
    /// # Arguments
    /// * `value`: The value to create the enum from
    ///
    /// # Returns
    /// Result of trying to create the enum.
    ///
    /// # Panics
    /// Will panic if attempting to convert an unsupported value to the enum
    fn from(value: u16) -> Self {
        match value {
            0b1100 => VemlIntegration::Int25,
            0b1000 => VemlIntegration::Int50,
            0b0000 => VemlIntegration::Int100,
            0b0001 => VemlIntegration::Int200,
            0b0010 => VemlIntegration::Int400,
            0b0011 => VemlIntegration::Int800,
            _ => panic!("Can not convert {value} into VemlIntegration"),
        }
    }
}

/// Enumeration of the persistence protection value of the sensor.
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum VemlPersistence {
    /// Persistence Protect 1
    Persist1 = 0x00,

    /// Persistence Protect 2
    Persist2 = 0x01,

    /// Persistence Protect 4
    Persist4 = 0x02,

    /// Persistence Protect 8
    Persist8 = 0x04,
}

impl From<u16> for VemlPersistence {
    /// Try to create enum from integer
    ///
    /// # Arguments
    /// * `value`: The value to create the enum from
    ///
    /// # Returns
    /// Result of trying to create the enum.
    fn from(value: u16) -> Self {
        match value {
            0b00 => VemlPersistence::Persist1,
            0b01 => VemlPersistence::Persist2,
            0b10 => VemlPersistence::Persist4,
            0b11 => VemlPersistence::Persist8,
            _ => panic!("Can not convert {value} into VemlPersistence"),
        }
    }
}

/// VEML 7700 Configuration Register Structure
#[derive(Clone, Copy, Debug)]
pub struct VemlConfigReg {
    /// The ALS channel gain
    pub gain: VemlGain,

    /// The ALS integration time.
    pub integration_time: VemlIntegration,

    /// ALS Channel Persistence Protection
    pub persistence: VemlPersistence,

    /// Whether or not to enable the ALS interrupt
    pub interrupt_enabled: bool,

    /// Whether or not the shut down the ALS sensor. Set to True to shut down.
    pub shutdown: bool,
}

impl From<u16> for VemlConfigReg {
    fn from(value: u16) -> Self {
        let shutdown = (value & 0x01) != 0;
        let interrupt_enabled = (value & 0x02) != 0;
        let persistence = VemlPersistence::try_from((value >> 4) & 0b11).unwrap();
        let integration_time = VemlIntegration::try_from((value >> 6) & 0b1111).unwrap();
        let gain = VemlGain::try_from((value >> 11) & 0b11).unwrap();

        Self {
            gain,
            integration_time,
            persistence,
            interrupt_enabled,
            shutdown,
        }
    }
}

impl From<VemlConfigReg> for u16 {
    fn from(value: VemlConfigReg) -> Self {
        let gain_int = value.gain as u16;
        let integration_int = value.integration_time as u16;
        let persist_int = value.persistence as u16;
        let interrupt_int = u16::from(value.interrupt_enabled);
        let shutdown_int = u16::from(value.shutdown);

        (gain_int << 11)
            | (integration_int << 6)
            | (persist_int << 4)
            | (interrupt_int << 1)
            | shutdown_int
    }
}

/// Structure of the output data from the sensor
#[derive(Clone, Copy, Debug, Default)]
pub struct VemlOutput {
    /// Raw ambient ligth sensor value.
    pub raw_als: u16,

    /// Raw white sensor value
    pub raw_white: u16,

    /// Computed Brightness in LUX
    pub lux: f32,
}

impl VemlOutput {
    /// Create a new empty instance of the structure.
    #[must_use]
    pub fn new() -> Self {
        Self {
            raw_als: 0,
            raw_white: 0,
            lux: 0.0,
        }
    }
}

/// Main structure for the VEML7700
pub struct Veml7700<I2C> {
    /// Concrete I2C implementation
    i2c: I2C,

    /// Sensor configuration
    configuration: VemlConfigReg,

    /// Values from the most recent sensor measurement.
    last_output: VemlOutput,
}

// TODO: Add method for computing refresh time and resolution given gain, power saving mode, integration time

impl<I2C: I2c> Veml7700<I2C> {
    /// Create a new instance of the VEML7700 driver.
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            configuration: VemlConfigReg {
                gain: VemlGain::Gain1,
                integration_time: VemlIntegration::Int100,
                persistence: VemlPersistence::Persist1,
                interrupt_enabled: false,
                shutdown: false,
            },
            last_output: VemlOutput {
                raw_als: 0,
                raw_white: 0,
                lux: 0.0,
            },
        }
    }

    /// Write configuration structure to the sensor
    ///
    /// # Arguments
    /// * `config`: The configuration to write to the sensor
    ///
    /// # Returns
    /// Result of writing the configuration to the sensor.
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    pub fn set_configuration(&mut self, config: VemlConfigReg) -> Result<(), I2C::Error> {
        self.configuration = config;
        self.write_internal_configuration()
    }

    /// Get the sensor configuration
    ///
    /// # Returns
    /// The read sensor configration, or an error
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    pub fn get_configuration(&mut self) -> Result<VemlConfigReg, I2C::Error> {
        let result = self.write_read_u16(VemlRegister::Config)?;
        Ok(VemlConfigReg::from(result))
    }

    /// Get the raw ALS value from the sensor
    ///
    /// # Returns
    /// Result of either the raw ALS value from the sensor, or an error code
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    pub fn get_ambient_level(&mut self) -> Result<u16, I2C::Error> {
        self.write_read_u16(VemlRegister::ALSValue)
    }

    /// Gets the white value from the sensor
    ///
    /// # Returns
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    /// The read white value, or an error code.
    pub fn get_white_level(&mut self) -> Result<u16, I2C::Error> {
        self.write_read_u16(VemlRegister::WhiteValue)
    }

    /// Get the computed ALS Lux Value
    ///
    /// # Returns
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    /// Computed ALS lux value
    pub fn get_lux(&mut self) -> Result<f32, I2C::Error> {
        let raw_als = self.get_ambient_level()?;
        Ok(self.get_als_scale() * f32::from(raw_als))
    }

    /// Set the sensor gain to the specified value.
    ///
    /// # Arugments
    /// * `gain` The gain to set
    ///
    /// # Returns
    /// Result of setting the gain
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    pub fn set_gain(&mut self, gain: VemlGain) -> Result<(), I2C::Error> {
        self.configuration.gain = gain;
        self.write_internal_configuration()
    }

    /// Get the sensor gain
    ///
    /// # Returns
    /// Result of getting the gain
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    pub fn get_gain(&mut self) -> Result<VemlGain, I2C::Error> {
        self.get_configuration()?;
        Ok(self.configuration.gain)
    }

    /// Set the sensor integration time
    ///
    /// # Arguments
    /// * `integration_time`: The integration time to set
    ///
    /// # Returns
    /// Result of setting the integration time
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    pub fn set_integration_time(
        &mut self,
        integration_time: VemlIntegration,
    ) -> Result<(), I2C::Error> {
        self.configuration.integration_time = integration_time;
        self.write_internal_configuration()
    }

    /// Get the sensor integration time
    ///
    /// # Returns
    /// The sensor integration time
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    pub fn get_intergration_time(&mut self) -> Result<VemlIntegration, I2C::Error> {
        self.get_configuration()?;
        Ok(self.configuration.integration_time)
    }

    /// Set the sensor power state
    ///
    /// # Arguments
    /// * `shutdown`: If true, sets the sensor to the shutdown power state
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    pub fn set_power_state(&mut self, shutdown: bool) -> Result<(), I2C::Error> {
        self.configuration.shutdown = shutdown;
        self.write_internal_configuration()
    }

    /// Get the sensor power state
    ///
    /// # Returns
    /// Resulting holding boolean. If the boolean is true, the sensor is
    /// powered down.
    ///
    /// # Errors
    /// Will return an error if the I2C Transation Fails
    pub fn get_power_state(&mut self) -> Result<bool, I2C::Error> {
        self.get_configuration()?;
        Ok(self.configuration.shutdown)
    }

    /// Get the ALS interrupt high threshold.
    ///
    /// # Returns
    /// The ALS Interrupt high threshold
    ///
    /// # Errors
    /// Will return an error if the I2C transaction fails.
    pub fn get_als_int_high_threshold(&mut self) -> Result<u16, I2C::Error> {
        self.write_read_u16(VemlRegister::ALSHighThreshold)
    }

    /// Set the ALS Interrupt high threshold.
    ///
    /// # Arguments
    /// * `threshold`: The value to set for the ALS interrupt high threshold
    ///
    /// # Errors
    /// Will return an error if the I2C transaction fails.
    pub fn set_als_int_high_threshold(&mut self, threshold: u16) -> Result<(), I2C::Error> {
        self.write_u16(VemlRegister::ALSHighThreshold, threshold)
    }

    /// Get the ALS interrupt low threshold.
    ///
    /// # Returns
    /// The ALS Interrupt low threshold
    ///
    /// # Errors
    /// Will return an error if the I2C transaction fails.
    pub fn get_als_int_low_threshold(&mut self) -> Result<u16, I2C::Error> {
        self.write_read_u16(VemlRegister::ALSLowThreshold)
    }

    /// Set the ALS Interrupt low threshold.
    ///
    /// # Arguments
    /// * `threshold`: The value to set for the ALS interrupt low threshold
    ///
    /// # Errors
    /// Will return an error if the I2C transaction fails.
    pub fn set_als_int_low_threshold(&mut self, threshold: u16) -> Result<(), I2C::Error> {
        self.write_u16(VemlRegister::ALSLowThreshold, threshold)
    }

    /// Get the interrupt status fo the sensor
    ///
    /// # Returns
    /// Tuple where the first element is a boolean indicating if the low
    /// threshold has been crosssed, and the second element is a boolean indicating
    /// if the high threshold has been crossed.
    ///
    /// # Errors
    /// Returns an error if reading the interrupt status register failed.
    pub fn get_interrupt_status(&mut self) -> Result<(bool, bool), I2C::Error> {
        let reg_value = self.write_read_u16(VemlRegister::ALSInterruptStatus)?;
        let low = reg_value & 8000;
        let high = reg_value & 4000;

        Ok((low > 0, high > 0))
    }

    /// Set the power saving mode of the sensor.
    ///
    /// # Arguments
    /// * `mode`: The power saving mode to use.
    ///
    /// # Errors
    /// Returns an error if setting the mode over I2C failed.
    pub fn set_power_saving_mode(&mut self, mode: VemlPowerSavingMode) -> Result<(), I2C::Error> {
        let enable = self.get_power_saving_status()?;
        self.write_power_save_reg(mode, enable)
    }

    /// Get the power saving mode of the sensor.
    ///
    /// # Returns
    /// Current set power saving mode of the sensor
    ///
    /// # Errors
    /// Returns an error if getting the power saving mode failed.
    ///
    /// # Panics
    /// Will panic if converting the mode bits into `VemlPowerSavingMode` fails
    pub fn get_power_saving_mode(&mut self) -> Result<VemlPowerSavingMode, I2C::Error> {
        let reg_val = self.write_read_u16(VemlRegister::PowerSaving)?;
        let mode_bits = (reg_val & 0b110) >> 1;

        Ok(mode_bits.into())
    }

    /// Enable/Disable use of the power saving mode.
    ///
    /// # Arguments
    /// * `enable`: Whether or not to enable power saving mode.
    ///
    /// # Errors
    /// Returns an error if enabling/disabling power saving failed.
    pub fn toggle_power_saving(&mut self, enable: bool) -> Result<(), I2C::Error> {
        let mode = self.get_power_saving_mode()?;
        self.write_power_save_reg(mode, enable)
    }

    /// Get whether or not power saving mode is enabled
    ///
    /// # Returns
    /// Booleaning indicating if the power saving mode is enabled
    ///
    /// # Errors
    /// Returrns an error if reading the power saving mode failed.
    pub fn get_power_saving_status(&mut self) -> Result<bool, I2C::Error> {
        let reg_val = self.write_read_u16(VemlRegister::PowerSaving)?;
        let mode_bits = reg_val & 0b01;
        Ok(mode_bits == 1)
    }

    /// Perform the VEML task's periodic prrocessing
    ///
    /// # Panics
    /// Will panic if reading the ambient level or white level fails
    pub fn periodic_process(&mut self) {
        let raw_als = self.get_ambient_level().unwrap();
        let raw_white = self.get_white_level().unwrap();
        let lux = f32::from(raw_als) * self.get_als_scale();
        self.last_output = VemlOutput {
            raw_als,
            raw_white,
            lux,
        }
    }

    /// Get the most recent set of data read from the sensor
    ///
    /// # Returns
    /// Most recently read data from the sensor
    pub fn get_outputs(&self) -> VemlOutput {
        self.last_output
    }

    /// Write to the power saving register
    ///
    /// # Arguments
    /// * `mode`: The mode to write to the power saving register
    /// * `enable`: The value to write to the enable bit
    fn write_power_save_reg(
        &mut self,
        mode: VemlPowerSavingMode,
        enable: bool,
    ) -> Result<(), I2C::Error> {
        let reg_value = ((mode as u16) << 1) | u16::from(enable);
        self.write_u16(VemlRegister::PowerSaving, reg_value)
    }

    /// Get the current scale factor based on gain and integration time
    fn get_als_scale(&self) -> f32 {
        let gain_scale: u16 = match self.configuration.gain {
            VemlGain::Gain2 => 1,
            VemlGain::Gain1 => 2,
            VemlGain::Gain1_4 => 8,
            VemlGain::Gain1_8 => 16,
        };

        let integration_scale: u16 = match self.configuration.integration_time {
            VemlIntegration::Int25 => 32,
            VemlIntegration::Int50 => 16,
            VemlIntegration::Int100 => 8,
            VemlIntegration::Int200 => 4,
            VemlIntegration::Int400 => 2,
            VemlIntegration::Int800 => 1,
        };

        ALS_BASE_SCALE * f32::from(gain_scale * integration_scale)
    }

    /// Write the VEML Configuration from the internal structure
    ///
    /// # Returns
    /// Result of writing the config
    fn write_internal_configuration(&mut self) -> Result<(), I2C::Error> {
        self.write_u16(VemlRegister::Config, self.configuration.into())
    }

    /// Read a u16 Data register
    ///
    /// # Arguments
    /// * `reg` The reagister to read.
    ///
    /// # Returns
    /// Result of reading the register, or an error code.
    fn write_read_u16(&mut self, reg: VemlRegister) -> Result<u16, I2C::Error> {
        let tx_buf = [reg as u8];
        let mut rx_buf = [0, 0];
        self.i2c.write_read(VEML_ADDR, &tx_buf, &mut rx_buf)?;
        let result = ((u16::from(rx_buf[1])) << 8) | (u16::from(rx_buf[0]));
        Ok(result)
    }

    /// Write to a u16 data register
    ///
    /// # Arguments
    /// * `reg`: The register to write to
    /// * `data`: The data to write to the register.
    ///
    /// # Returns
    /// Result of writing to the register.
    fn write_u16(&mut self, reg: VemlRegister, data: u16) -> Result<(), I2C::Error> {
        let data_bytes = data.to_le_bytes();
        let tx_buf = [reg as u8, data_bytes[0], data_bytes[1]];
        self.i2c.write(VEML_ADDR, &tx_buf)
    }
}
