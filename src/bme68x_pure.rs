//! BME68X Driver Implementation in pure rust.
// TODO: More enumerations to replace constants

// Other stuff
const BME68X_CHIP_ID: u8 = 0x61;
const BME68X_SOFT_RESET_CMD: u8 = 0xb6;
const BME68X_ENABLE: u8 = 0x01;

// For self test
const BME68X_HEATR_DUR1: u16 = 1000;
const BME68X_HEATR_DUR2: u16 = 2000;
const BME68X_LOW_TEMP: u16 = 150;
const BME68X_HIGH_TEMP: u16 = 350;
const BME68X_N_MEAS: usize = 6;
const BME68X_HEATR_DUR1_DELAY: u32 = 1000000;
const BME68X_HEATR_DUR2_DELAY: u32 = 2000000;

// TODO: Make these an enum
const BME68X_DISABLE_HEATER: u8 = 0x01;
const BME68X_ENABLE_HEATER: u8 = 0x00;

// TODO: Make these an enum
const BME68X_DISABLE_GAS_MEAS: u8 = 0x00;
const BME68X_ENABLE_GAS_MEAS_H: u8 = 0x02;
const BME68X_ENABLE_GAS_MEAS_L: u8 = 0x01;

// TODO: Make these an enum
const BME68X_VARIANT_GAS_LOW: u32 = 0x00;
const BME68X_VARIANT_GAS_HIGH: u32 = 0x01;

// Min/max values allowed.
const BME68X_MIN_TEMPERATURE: f32 = 0.0;
const BME68X_MAX_TEMPERATURE: f32 = 60.0;
const BME68X_MIN_PRESSURE: f32 = 90000.0;
const BME68X_MAX_PRESSURE: f32 = 110000.0;
const BME68X_MIN_HUMIDITY: f32 = 20.0;
const BME68X_MAX_HUMIDITY: f32 = 80.0;

// Masks

/// Mask for number of conversions
const BME68X_NBCONV_MSK: u8 = 0x0f;

/// Mask for IIR filter
const BME68X_FILTER_MSK: u8 = 0x1c;

/// Mask for ODR[3]
const BME68X_ODR3_MSK: u8 = 0x80;

/// Mask for ODR[2:0]
const BME68X_ODR20_MSK: u8 = 0xe0;

/// Mask for temperature oversampling
const BME68X_OST_MSK: u8 = 0xe0;

/// Mask for pressure oversampling
const BME68X_OSP_MSK: u8 = 0x1c;

/// Mask for humidity oversampling
const BME68X_OSH_MSK: u8 = 0x07;

/// Mask for heater control
const BME68X_HCTRL_MSK: u8 = 0x08;

/// Mask for run gas
const BME68X_RUN_GAS_MSK: u8 = 0x30;

/// Mask for operation mode
const BME68X_MODE_MSK: u8 = 0x03;

/// Mask for res heat range
const BME68X_RHRANGE_MSK: u8 = 0x30;

/// Mask for range switching error
const BME68X_RSERROR_MSK: u8 = 0xf0;

/// Mask for new data
const BME68X_NEW_DATA_MSK: u8 = 0x80;

/// Mask for gas index
const BME68X_GAS_INDEX_MSK: u8 = 0x0f;

/// Mask for gas range
const BME68X_GAS_RANGE_MSK: u8 = 0x0f;

/// Mask for gas measurement valid
const BME68X_GASM_VALID_MSK: u8 = 0x20;

/// Mask for heater stability
const BME68X_HEAT_STAB_MSK: u8 = 0x10;

/// Mask for SPI memory page
const BME68X_MEM_PAGE_MSK: u8 = 0x10;

/// Mask for reading a register in SPI
const BME68X_SPI_RD_MSK: u8 = 0x80;

/// Mask for writing a register in SPI
const BME68X_SPI_WR_MSK: u8 = 0x7f;

/// Mask for the H1 calibration coefficient
const BME68X_BIT_H1_DATA_MSK: u16 = 0x0f;

//  Coefficient index macros

///  Length for all coefficients
const BME68X_LEN_COEFF_ALL: usize = 42;

///  Length for 1st group of coefficients
const BME68X_LEN_COEFF1: usize = 23;

///  Length for 2nd group of coefficients
const BME68X_LEN_COEFF2: usize = 14;

///  Length for 3rd group of coefficients
const BME68X_LEN_COEFF3: usize = 5;

///  Length of the field
const BME68X_LEN_FIELD: u8 = 17;

///  Length between two fields
const BME68X_LEN_FIELD_OFFSET: u8 = 17;

///  Length of the configuration register
const BME68X_LEN_CONFIG: usize = 5;

///  Length of the interleaved buffer
const BME68X_LEN_INTERLEAVE_BUFF: u8 = 20;

//  Coefficient index macros

///  Coefficient T2 LSB position
const BME68X_IDX_T2_LSB: usize = 0;

///  Coefficient T2 MSB position
const BME68X_IDX_T2_MSB: usize = 1;

///  Coefficient T3 position
const BME68X_IDX_T3: usize = 2;

///  Coefficient P1 LSB position
const BME68X_IDX_P1_LSB: usize = 4;

///  Coefficient P1 MSB position
const BME68X_IDX_P1_MSB: usize = 5;

///  Coefficient P2 LSB position
const BME68X_IDX_P2_LSB: usize = 6;

///  Coefficient P2 MSB position
const BME68X_IDX_P2_MSB: usize = 7;

///  Coefficient P3 position
const BME68X_IDX_P3: usize = 8;

///  Coefficient P4 LSB position
const BME68X_IDX_P4_LSB: usize = 10;

///  Coefficient P4 MSB position
const BME68X_IDX_P4_MSB: usize = 11;

///  Coefficient P5 LSB position
const BME68X_IDX_P5_LSB: usize = 12;

///  Coefficient P5 MSB position
const BME68X_IDX_P5_MSB: usize = 13;

///  Coefficient P7 position
const BME68X_IDX_P7: usize = 14;

///  Coefficient P6 position
const BME68X_IDX_P6: usize = 15;

///  Coefficient P8 LSB position
const BME68X_IDX_P8_LSB: usize = 18;

///  Coefficient P8 MSB position
const BME68X_IDX_P8_MSB: usize = 19;

///  Coefficient P9 LSB position
const BME68X_IDX_P9_LSB: usize = 20;

///  Coefficient P9 MSB position
const BME68X_IDX_P9_MSB: usize = 21;

///  Coefficient P10 position
const BME68X_IDX_P10: usize = 22;

///  Coefficient H2 MSB position
const BME68X_IDX_H2_MSB: usize = 23;

///  Coefficient H2 LSB position
const BME68X_IDX_H2_LSB: usize = 24;

///  Coefficient H1 LSB position
const BME68X_IDX_H1_LSB: usize = 24;

///  Coefficient H1 MSB position
const BME68X_IDX_H1_MSB: usize = 25;

///  Coefficient H3 position
const BME68X_IDX_H3: usize = 26;

///  Coefficient H4 position
const BME68X_IDX_H4: usize = 27;

///  Coefficient H5 position
const BME68X_IDX_H5: usize = 28;

///  Coefficient H6 position
const BME68X_IDX_H6: usize = 29;

///  Coefficient H7 position
const BME68X_IDX_H7: usize = 30;

///  Coefficient T1 LSB position
const BME68X_IDX_T1_LSB: usize = 31;

///  Coefficient T1 MSB position
const BME68X_IDX_T1_MSB: usize = 32;

///  Coefficient GH2 LSB position
const BME68X_IDX_GH2_LSB: usize = 33;

///  Coefficient GH2 MSB position
const BME68X_IDX_GH2_MSB: usize = 34;

///  Coefficient GH1 position
const BME68X_IDX_GH1: usize = 35;

///  Coefficient GH3 position
const BME68X_IDX_GH3: usize = 36;

///  Coefficient res heat value position
const BME68X_IDX_RES_HEAT_VAL: usize = 37;

///  Coefficient res heat range position
const BME68X_IDX_RES_HEAT_RANGE: usize = 39;

///  Coefficient range switching error position
const BME68X_IDX_RANGE_SW_ERR: usize = 41;

/// Filter bit position
const BME68X_FILTER_POS: u8 = 2;

/// Temperature oversampling bit position
const BME68X_OST_POS: u8 = 5;

/// Pressure oversampling bit position
const BME68X_OSP_POS: u8 = 2;

/// ODR[3] bit position
const BME68X_ODR3_POS: u8 = 7;

/// ODR[2:0] bit position
const BME68X_ODR20_POS: u8 = 5;

/// Run gas bit position
const BME68X_RUN_GAS_POS: u8 = 4;

/// Heater control bit position
const BME68X_HCTRL_POS: u8 = 3;

/// BME68x error codes
#[repr(i8)]
#[derive(Debug, Clone, Copy)]
pub enum BME68xError {
    /// Success
    Ok = 0,

    /// Null Pointer Passed
    NullPtr = -1,

    /// Communication Failure
    ComFail = -2,

    /// Sensor Not Found
    DevNotFound = -3,

    /// Incorrect Length Parameter
    InvalidLength = -4,

    /// Self Test Error
    SelfTest = -5,

    /// Define a valid operation mode
    DefineOpMode = 1,

    /// No New Data was found
    NoNewData = 2,

    /// Define shared heating duration
    DefineShdHeatrDur = 3,
}

/// BME68X Registers
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BME68xRegister {
    ///Register for 3rd group of coefficients
    Coeff3 = 0x00,

    /// 0th Field address
    Field0 = 0x1d,

    /// 0th Current DAC address
    IdacHeat0 = 0x50,

    /// 0th Res heat address
    ResHeat0 = 0x5a,

    /// 0th Gas wait address
    GasWait0 = 0x64,

    /// Shared heating duration address
    ShdHeatrDur = 0x6E,

    /// CTRLGAS0 address
    CtrlGas0 = 0x70,

    /// CTRLGAS1 address
    CtrlGas1 = 0x71,

    /// CTRLHUM address
    CtrlHum = 0x72,

    /// CTRLMEAS address
    CtrlMeas = 0x74,

    /// CONFIG address
    Config = 0x75,

    /// MEMPAGE address
    MemPage = 0xf3,

    /// Unique ID address
    UniqueId = 0x83,

    /// Register for 1st group of coefficients
    Coeff1 = 0x8a,

    /// Chip ID address
    ChipId = 0xd0,

    /// Soft reset address
    SoftReset = 0xe0,

    /// Register for 2nd group of coefficients
    Coeff2 = 0xe1,

    /// Variant ID Register
    VariantId = 0xF0,
}

/// BME68X Oversampling Settings
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BME68xOs {
    /// Switch off measurement
    OsNone = 0,

    /// Perform 1 measurement
    Os1x = 1,

    /// Perform 2 measurements
    Os2x = 2,

    /// Perform 4 measurements
    Os4x = 3,

    /// Perform 8 measurements
    Os8x = 4,

    /// Perform 16 measurements
    Os16x = 5,
}

impl From<u8> for BME68xOs {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::OsNone,
            1 => Self::Os1x,
            2 => Self::Os2x,
            3 => Self::Os4x,
            4 => Self::Os8x,
            5 => Self::Os16x,
            _ => panic!("Could not convert {value} into BME68xOs"),
        }
    }
}

/// Enumeration of the ODR/standby times
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum BME68xODR {
    ///  Standby time of 0.59ms
    ODR059Ms = 0,

    ///  Standby time of 62.5ms
    ODR625Ms = 1,

    ///  Standby time of 125ms
    ODR125Ms = 2,

    ///  Standby time of 250ms
    ODR250Ms = 3,

    ///  Standby time of 500ms
    ODR500Ms = 4,

    ///  Standby time of 1s
    ODR1000Ms = 5,

    ///  Standby time of 10ms
    ODR10Ms = 6,

    ///  Standby time of 20ms
    ODR20Ms = 7,

    ///  No standby time
    ODRNone = 8,
}

impl From<u8> for BME68xODR {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::ODR059Ms,
            1 => Self::ODR625Ms,
            2 => Self::ODR125Ms,
            3 => Self::ODR250Ms,
            4 => Self::ODR500Ms,
            5 => Self::ODR1000Ms,
            6 => Self::ODR10Ms,
            7 => Self::ODR20Ms,
            8 => Self::ODRNone,
            _ => panic!("Cannot convert {value} into BME68xODR"),
        }
    }
}

// TODO: Conditional FPU support?

/// Enumeration of possible interfaces for the sensor.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BME68xIntf {
    /// SPI Interface
    SPIIntf = 0,

    /// I2C Interface
    I2CIntf = 1,
}

/// BME68X Operating Modes
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BME68xOpMode {
    /// Sleep Mode
    SleepMode = 0,

    /// Forced Mode
    ForcedMode = 1,

    /// Parallel mode
    ParallelMode = 2,

    /// Sequential Mode
    SequentialMode = 3,
}

impl From<u8> for BME68xOpMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::SleepMode,
            1 => Self::ForcedMode,
            2 => Self::ParallelMode,
            3 => Self::SequentialMode,
            _ => panic!("Cannot Convert {value} to a BME68xOpMode"),
        }
    }
}

/// Sensor Field Data Structure
#[derive(Clone, Copy)]
pub struct BME68xData {
    /// Sensor Status (new_data, gasm_valid, heat_stab)
    pub status: u8,

    /// Index of the heater profile in use
    pub gas_index: u8,

    /// Measurement Index to track order
    pub meas_index: u8,

    ///Heater resistance
    pub res_heat: u8,

    /// Current DAC
    pub idac: u8,

    /// Gas wait period
    pub gas_wait: u8,

    /// Temperature in degrees celsius
    pub temperature: f32,

    /// Pressure in Pascal
    pub pressure: f32,

    /// Humidity in % relative humidity x1000
    pub humidity: f32,

    /// Gas Resistance in Ohms
    pub gas_resistance: f32,
}

impl BME68xData {
    /// Create a new instance with all members set to 0
    pub fn new() -> Self {
        BME68xData {
            status: 0,
            gas_index: 0,
            meas_index: 0,
            res_heat: 0,
            idac: 0,
            gas_wait: 0,
            temperature: 0.0,
            pressure: 0.0,
            humidity: 0.0,
            gas_resistance: 0.0,
        }
    }
}

/// Calibration Coefficent Data Structurre
#[derive(Clone, Copy)]
pub struct BME68xCalibData {
    /// Calibration coefficient for the humidity sensor
    pub par_h1: u16,

    /// Calibration coefficient for the humidity sensor
    pub par_h2: u16,

    /// Calibration coefficient for the humidity sensor
    pub par_h3: i8,

    /// Calibration coefficient for the humidity sensor
    pub par_h4: i8,

    /// Calibration coefficient for the humidity sensor
    pub par_h5: i8,

    /// Calibration coefficient for the humidity sensor
    pub par_h6: u8,

    /// Calibration coefficient for the humidity sensor
    pub par_h7: i8,

    /// Calibration coefficient for the gas sensor
    pub par_gh1: i8,

    /// Calibration coefficient for the gas sensor
    pub par_gh2: i16,

    /// Calibration coefficient for the gas sensor
    pub par_gh3: i8,

    /// Calibration coefficient for the temperature sensor
    pub par_t1: u16,

    /// Calibration coefficient for the temperature sensor
    pub par_t2: i16,

    /// Calibration coefficient for the temperature sensor
    pub par_t3: i8,

    /// Calibration coefficient for the pressure sensor
    pub par_p1: u16,

    /// Calibration coefficient for the pressure sensor
    pub par_p2: i16,

    /// Calibration coefficient for the pressure sensor
    pub par_p3: i8,

    /// Calibration coefficient for the pressure sensor
    pub par_p4: i16,

    /// Calibration coefficient for the pressure sensor
    pub par_p5: i16,

    /// Calibration coefficient for the pressure sensor
    pub par_p6: i8,

    /// Calibration coefficient for the pressure sensor
    pub par_p7: i8,

    /// Calibration coefficient for the pressure sensor
    pub par_p8: i16,

    /// Calibration coefficient for the pressure sensor
    pub par_p9: i16,

    /// Calibration coefficient for the pressure sensor
    pub par_p10: u8,

    /// Variable to store the intermediate temperature coefficient
    pub t_fine: f32,

    /// Heater resistance range coefficient
    pub res_heat_range: u8,

    /// Heater resistance value coefficient
    pub res_heat_val: i8,

    /// Gas resistance range switching error coefficient
    pub range_sw_err: i8,
}

/// BME68X sensor settings structure which comprises of ODR, over-sampling and filter settings.
#[derive(Clone, Copy)]
pub struct BME68xConf {
    /// Humidity overrsampling
    pub os_hum: BME68xOs,

    /// Temperature Oversampling
    pub os_temp: BME68xOs,

    /// Pressure Oversampling
    pub os_pres: BME68xOs,

    /// Filter Coefficent
    pub filter: u8,

    /// Standby time between sequential mode measurement profiles
    pub odr: BME68xODR,
}

/// Gas Heater Configuration
pub struct BME68xHeatrConf {
    /// Enable gas measurement
    pub enable: u8,

    /// Store the heater temperature for forced mode degree Celsius
    pub heatr_temp: u16,

    /// Store the heating duration for forced mode in milliseconds
    pub heatr_dur: u16,

    /// Store the heater temperature profile in degree Celsius, Max of 10
    pub heatr_temp_prof: [u16; 10],

    /// Store the heating duration profile in milliseconds. Max of 10
    pub heatr_dur_prof: [u16; 10],

    /// Variable to store the length of the heating profile
    // FIXME: Can probably remove since vec can have length calcualted
    pub profile_len: u8,

    /// Variable to store heating duration for parallel mode in milliseconds
    pub shared_heatr_dur: u16,
}

impl BME68xHeatrConf {
    /// Create a new empty instance.
    pub fn new() -> Self {
        Self {
            enable: 0,
            heatr_temp: 0,
            heatr_dur: 0,
            heatr_temp_prof: [0; 10],
            heatr_dur_prof: [0; 10],
            profile_len: 0,
            shared_heatr_dur: 0,
        }
    }
}

/// BME68X Device Structure
#[derive(Clone, Copy)]
pub struct BME68xDev {
    /// Chip ID
    chip_id: u8,

    /// The interface pointer is used to enable the user to link their
    /// interface descriptors for reference during the implementation of
    /// the read and write interfaces to the hardware.
    intf_ptr: u8, // TODO: Figure out what to do here!

    /// Variant ID.
    /// 0 = BME68X_VARIANT_GAS_LOW
    /// 1 = BME68X_VARIANT_GAS_HIGH
    variant_id: u32,

    /// SPI/I2C Interface
    intf: BME68xIntf,

    /// Memory page used
    mem_page: u8,

    /// Ambient Temperature in degrees C
    amb_temp: i8,

    /// Sensor Calibration Data
    calib: BME68xCalibData,

    //TODO: bme68x_read_fptr_t read;
    // TODO: bme68x_write_fptr_t write;
    // TOOD:  bme68x_delay_us_fptr_t delay_us
    /// To store interface pointer error
    intf_rslt: i8,

    /// Store info messages
    info_msg: BME68xError,
}

impl BME68xDev {
    /// Initialize the sensor.
    ///
    /// Reads the Chip ID and calibrates the sensor.
    /// This should be called before all other functions.
    ///
    /// # Errors
    /// Returns an error if the initialzation is unsuccessful.
    pub fn init(&mut self) -> Result<(), BME68xError> {
        self.soft_reset().unwrap();

        self.chip_id = self.get_regs(BME68xRegister::ChipId, 1)?[0];
        if self.chip_id == BME68X_CHIP_ID {
            self.read_variant_id()
        } else {
            Err(BME68xError::DevNotFound)
        }
    }

    /// Write the given data to teh registers address of the sensor.
    ///
    /// # Arguments
    /// * `reg_addr`: Register addresess to write data to
    /// * `reg_data`: Data to write to the registers
    /// * `len`: Number of bytes of data to write.
    ///
    /// # Errors
    /// Errors if failing to write to the registers.
    // FIXME: Use the enum for registers. Will cause issues with the setting of heater conf
    pub fn set_regs(
        &self,
        reg_addr: &[u8],
        reg_data: &[u8],
        len: usize,
    ) -> Result<(), BME68xError> {
        todo!();
    }

    /// Read data from the given registers
    ///
    /// # Arguments
    /// * `reg_addr`: Register addresses to read data from
    /// * `len`: Number of bytes of data to read
    ///
    /// # Returns
    /// Data read from the registers
    ///
    /// # Errors
    /// Errors if failing to read from the registers.
    // TODO: Remove the len argument?
    pub fn get_regs(&self, reg_addr: BME68xRegister, len: usize) -> Result<&[u8], BME68xError> {
        todo!();
    }

    /// Soft-Reset the sensorr
    ///
    /// # Errors
    /// Returns an error if soft-resetting the sensor failed.
    pub fn soft_reset(&self) -> Result<(), BME68xError> {
        self.get_mem_page()?;
        if (matches!(self.intf, BME68xIntf::SPIIntf)) {
            self.set_regs(
                &[BME68xRegister::SoftReset as u8],
                &[BME68X_SOFT_RESET_CMD],
                1,
            )?;
            todo!("dev->delay_us(BME68X_PERIOD_RESET, dev->intf_ptr);");
            self.get_mem_page()?;
        }
        Ok(())
    }

    /// Set the operation mode of the sensor
    ///
    /// # Arguments
    /// * `op_mode`: The desired operation mode
    ///
    /// # Errors
    /// Returns an error if setting the operation mode fails.
    pub fn set_op_mode(&self, op_mode: BME68xOpMode) -> Result<(), BME68xError> {
        let mut tmp_pow_mode;
        loop {
            tmp_pow_mode = self.get_regs(BME68xRegister::CtrlMeas, 1)?[0];
            let pow_mode: BME68xOpMode = (tmp_pow_mode & BME68X_MODE_MSK).into();

            if !matches!(pow_mode, BME68xOpMode::SleepMode) {
                // In rust ! is bitwise not
                tmp_pow_mode &= !BME68X_MODE_MSK; /* Set to sleep */
                self.set_regs(&[BME68xRegister::CtrlMeas as u8], &[tmp_pow_mode], 1)?;
                todo!("dev->delay_us(BME68X_PERIOD_POLL, dev->intf_ptr)");
            } else {
                break;
            }
        }
        /* Already in sleep */
        if !matches!(op_mode, BME68xOpMode::SleepMode) {
            tmp_pow_mode = (tmp_pow_mode & !BME68X_MODE_MSK) | (op_mode as u8 & BME68X_MODE_MSK);
            self.set_regs(&[BME68xRegister::CtrlMeas as u8], &[tmp_pow_mode], 1)?;
        }
        Ok(())
    }

    /// Get the operation mode of the sensor
    ///
    /// # Returns
    /// The current operation mode of the sensor
    ///
    /// # Errors
    /// Returns an error if getting the operation mode fails
    pub fn get_op_mode(&self) -> Result<BME68xOpMode, BME68xError> {
        let output = self.get_regs(BME68xRegister::CtrlMeas, 1)?[0];
        Ok(BME68xOpMode::from(output & BME68X_MODE_MSK))
    }

    /// Get the remaining duration that can be used for heating
    ///
    /// # Arguments
    /// * `op_mode`: The operation mode of the sensor
    /// * `conf`: The sensor configuration.
    pub fn get_meas_dur(&self, op_mode: BME68xOpMode, conf: &BME68xConf) -> i32 {
        let mut meas_dur;
        let mut meas_cycles;
        let os_to_meas_cycles = [0, 1, 2, 4, 8, 16];

        // TODO: Saferr way than using "as"
        meas_cycles = os_to_meas_cycles[conf.os_temp as usize];
        meas_cycles += os_to_meas_cycles[conf.os_pres as usize];
        meas_cycles += os_to_meas_cycles[conf.os_hum as usize];

        // TPH Measurement Duration
        meas_dur = meas_cycles * 1963;
        meas_dur += 477 * 5; // TPH Switching Duration
        meas_dur += 477 * 5; // Gas measurement duration

        if matches!(op_mode, BME68xOpMode::ParallelMode) {
            meas_dur += 1000; // Wake up diration of 1 ms
        }

        meas_dur
    }

    /// Read the pressure, temperature, humidity, and gas data from the sensor
    /// Then apply compensation to the data.
    ///
    /// # Arguments
    /// * `op_mode`: The operation mode of the sensor
    ///
    /// # Returns
    /// Tuple wheter the firsst element is the sensor data, and the second is the
    /// number of read elements
    ///
    /// # Errors
    // TODO: Remove the number of elements?
    pub fn get_data(&self, op_mode: BME68xOpMode) -> Result<([BME68xData; 3], u8), BME68xError> {
        let mut new_fields = 0;
        let mut data = [BME68xData::new(); 3];
        match op_mode {
            BME68xOpMode::ForcedMode => {
                self.read_field_data(0, &mut data)?;
                new_fields = 1;
            }
            BME68xOpMode::ParallelMode | BME68xOpMode::SequentialMode => {
                self.read_all_field_data(&mut data)?;

                // TODO: Check over this. Probably a way to do it properly in rust.
                // Sort sensor data
                for i in 0..3 {
                    if (data[i].status & BME68X_NEW_DATA_MSK) != 0 {
                        new_fields += 1;
                        for i in 0..2 {
                            for j in i + 1..3 {
                                sort_sensor_data(i, j, &mut data);
                            }
                        }
                    }
                }
            }
            _ => return Err(BME68xError::DefineOpMode),
        }
        if new_fields == 0 {
            Err(BME68xError::NoNewData)
        } else {
            Ok((data, new_fields))
        }
    }

    /// Set the oversampling, filter, and odr configuration
    ///
    /// Arguments
    /// * `conf`: Sensor Configuration.
    ///
    /// # Errors
    /// Returns an error if setting the configration failed.
    pub fn set_config(&mut self, conf: &BME68xConf) -> Result<(), BME68xError> {
        let mut odr20 = 0;
        let mut odr3 = 1;
        let reg_array = [0x71, 0x72, 0x73, 0x74, 0x75];
        let mut data_array = [0; BME68X_LEN_CONFIG];
        let current_op_mode = self.get_op_mode()?;

        // Configure only in sleep mode
        self.set_op_mode(BME68xOpMode::SleepMode)?;

        let curr_config = self.get_regs(BME68xRegister::CtrlGas1, BME68X_LEN_CONFIG)?;
        for i in 0..BME68X_LEN_CONFIG {
            data_array[i] = curr_config[i];
        }
        self.info_msg = BME68xError::Ok;
        data_array[4] = set_bits(
            data_array[4],
            BME68X_FILTER_MSK,
            BME68X_FILTER_POS,
            conf.filter as u8,
        );
        data_array[3] = set_bits(
            data_array[3],
            BME68X_OST_MSK,
            BME68X_OST_POS,
            conf.os_temp as u8,
        );
        data_array[3] = set_bits(
            data_array[3],
            BME68X_OSP_MSK,
            BME68X_OSP_POS,
            conf.os_pres as u8,
        );
        data_array[1] = set_bits_pos_0(data_array[1], BME68X_OSH_MSK, conf.os_hum as u8);
        if !matches!(conf.odr, BME68xODR::ODRNone) {
            odr20 = conf.odr as u8;
            odr3 = 0;
        }
        data_array[4] = set_bits(data_array[4], BME68X_ODR20_MSK, BME68X_ODR20_POS, odr20);
        data_array[0] = set_bits(data_array[0], BME68X_ODR3_MSK, BME68X_ODR3_POS, odr3);

        self.set_regs(&reg_array, &data_array, BME68X_LEN_CONFIG)?;
        self.set_op_mode(current_op_mode)
    }

    /// Get the oversampleing, filter, and odr configuration
    ///
    /// # Returns
    /// The current sensor configuration.
    ///
    /// # Errors
    /// Returns an error if getting the configuration failed.
    pub fn get_config(&self) -> Result<BME68xConf, BME68xError> {
        let data_array = self.get_regs(BME68xRegister::CtrlGas1, 5)?;
        Ok(BME68xConf {
            os_hum: BME68xOs::from(data_array[1] & BME68X_OSH_MSK),
            filter: get_bits(data_array[4], BME68X_FILTER_MSK, BME68X_FILTER_POS),
            os_temp: BME68xOs::from(get_bits(data_array[3], BME68X_OST_MSK, BME68X_OST_POS)),
            os_pres: BME68xOs::from(get_bits(data_array[3], BME68X_OSP_MSK, BME68X_OSP_POS)),
            odr: if get_bits(data_array[0], BME68X_ODR3_MSK, BME68X_ODR3_POS) == 0 {
                BME68xODR::ODRNone
            } else {
                BME68xODR::from(get_bits(data_array[4], BME68X_ODR20_MSK, BME68X_ODR20_POS))
            },
        })
    }

    /// Set the gas configuration of the sensor
    ///
    /// # Arguments
    /// * `op_mode` Expected operation mode of the sensor
    /// * `conf`: Desired heating configuration.
    ///
    /// # Errors
    /// Returns an error if seting the heater configuration failed
    pub fn set_heatr_conf(
        &self,
        op_mode: BME68xOpMode,
        conf: &BME68xHeatrConf,
    ) -> Result<(), BME68xError> {
        self.set_op_mode(BME68xOpMode::SleepMode)?;
        let mut hctrl = 0;
        let mut run_gas = 0;
        let mut ctrl_gas_data = [0; 2];
        let ctrl_gas_addr = [
            BME68xRegister::CtrlGas0 as u8,
            BME68xRegister::CtrlGas1 as u8,
        ];

        let nb_conv = self.set_conf(conf, op_mode)?;
        let gas_regs = self.get_regs(BME68xRegister::CtrlGas0, 2)?;
        for i in 0..2 {
            ctrl_gas_data[i] = gas_regs[i];
        }
        if conf.enable == BME68X_ENABLE {
            hctrl = BME68X_ENABLE_HEATER;
            if self.variant_id == BME68X_VARIANT_GAS_HIGH {
                run_gas = BME68X_ENABLE_GAS_MEAS_H;
            } else {
                run_gas = BME68X_ENABLE_GAS_MEAS_L;
            }
        } else {
            hctrl = BME68X_DISABLE_HEATER;
            run_gas = BME68X_DISABLE_GAS_MEAS;
        }

        ctrl_gas_data[0] = set_bits(ctrl_gas_data[0], BME68X_HCTRL_MSK, BME68X_HCTRL_POS, hctrl);
        ctrl_gas_data[1] = set_bits_pos_0(ctrl_gas_data[1], BME68X_NBCONV_MSK, nb_conv);
        ctrl_gas_data[1] = set_bits(
            ctrl_gas_data[1],
            BME68X_RUN_GAS_MSK,
            BME68X_RUN_GAS_POS,
            run_gas,
        );

        self.set_regs(&ctrl_gas_addr, &ctrl_gas_data, 2)
    }

    /// Get the heater configuration of the sensor
    ///
    ///
    /// # Returns
    /// The current heater configuration of the sensor
    ///
    /// # Errors
    /// Returns an error if reading the heater configuration failed.
    pub fn get_heatr_conf(&self) -> Result<BME68xHeatrConf, BME68xError> {
        let mut conf = BME68xHeatrConf::new();
        let temp_reg_data = self.get_regs(BME68xRegister::ResHeat0, 10)?;
        // FIXME: Pass in profile len conf, like in the original API.
        for i in 0..10 {
            conf.heatr_temp_prof[i] = temp_reg_data[i] as u16;
        }

        let time_reg_data = self.get_regs(BME68xRegister::GasWait0, 10)?;
        // FIXME: Pass in profile len conf, like in the original API.
        for i in 0..10 {
            conf.heatr_dur_prof[i] = time_reg_data[i] as u16;
        }
        Ok(conf)
    }

    /// Perform a self test of the low gas variant of the BME68x
    ///
    /// # Errors
    /// Returns an error if the self test failed.
    pub fn selftest_check(&self) -> Result<(), BME68xError> {
        let mut t_dev = self.clone();
        let conf = BME68xConf {
            os_hum: BME68xOs::Os1x,
            os_pres: BME68xOs::Os16x,
            os_temp: BME68xOs::Os2x,
            filter: 0,
            odr: BME68xODR::from(0),
        };
        let mut heatr_conf = BME68xHeatrConf {
            enable: BME68X_ENABLE,
            heatr_dur: BME68X_HEATR_DUR1,
            heatr_temp: BME68X_HIGH_TEMP,
            heatr_dur_prof: [0; 10],
            heatr_temp_prof: [0; 10],
            profile_len: 0,
            shared_heatr_dur: 0,
        };

        t_dev.set_config(&conf)?;
        t_dev.set_op_mode(BME68xOpMode::ForcedMode)?;
        // TODO: t_dev.delay_us(BME68X_HEATR_DUR1_DELAY, t_dev.intf_ptr);
        let (data, _) = t_dev.get_data(BME68xOpMode::ForcedMode)?;
        if !((data[0].idac != 0x00)
            && (data[0].idac != 0xFF)
            && ((data[0].status & BME68X_GASM_VALID_MSK) == 0))
        {
            return Err(BME68xError::SelfTest);
        }

        heatr_conf.heatr_dur = BME68X_HEATR_DUR2;

        let mut data = [BME68xData::new(); 3];
        let mut i = 0;
        while (i < BME68X_N_MEAS) {
            if (i % 2) == 0 {
                heatr_conf.heatr_temp = BME68X_HIGH_TEMP;
            } else {
                heatr_conf.heatr_temp = BME68X_LOW_TEMP;
            }
            t_dev.set_heatr_conf(BME68xOpMode::ForcedMode, &heatr_conf)?;
            t_dev.set_config(&conf)?;
            t_dev.set_op_mode(BME68xOpMode::ForcedMode)?;
            // TODO: t_dev.delay_us(BME68X_HEATR_DUR2_DELAY, t_dev.intf_ptr);
            (data, _) = t_dev.get_data(BME68xOpMode::ForcedMode)?;
            i += 1;
        }
        analyze_sensor_data(&data, BME68X_N_MEAS)
    }

    /*------------------------------------------------------------
     *                       Private Functions
     *-----------------------------------------------------------*/

    /// Read the calibration coefficents
    ///
    /// # Errors
    /// Errors if reading the calibration data failed.
    fn get_calib_data(&mut self) -> Result<(), BME68xError> {
        let mut coeff_array = [0; BME68X_LEN_COEFF_ALL];

        // Read first portion of the coefficent array.
        let result = self.get_regs(BME68xRegister::Coeff1, BME68X_LEN_COEFF1)?;
        for i in 0..BME68X_LEN_COEFF1 {
            coeff_array[i] = result[i];
        }

        // Read second chunk of coefficents.
        let result = self.get_regs(BME68xRegister::Coeff2, BME68X_LEN_COEFF2)?;
        for i in 0..BME68X_LEN_COEFF2 {
            coeff_array[i + BME68X_LEN_COEFF1] = result[i];
        }

        // Read the third chunk of coefficents
        let result = self.get_regs(BME68xRegister::Coeff3, BME68X_LEN_COEFF3)?;
        for i in 0..BME68X_LEN_COEFF3 {
            coeff_array[i + BME68X_LEN_COEFF1 + BME68X_LEN_COEFF2] = result[i];
        }

        // Copy data over
        /* Temperature related coefficients */
        // FIXME: The unsigned to signed conversions might not properly wrap
        self.calib.par_t1 = concat_bytes(
            coeff_array[BME68X_IDX_T1_MSB],
            coeff_array[BME68X_IDX_T1_LSB],
        );
        self.calib.par_t2 = concat_bytes(
            coeff_array[BME68X_IDX_T2_MSB],
            coeff_array[BME68X_IDX_T2_LSB],
        ) as i16;
        self.calib.par_t3 = coeff_array[BME68X_IDX_T3] as i8;

        /* Pressure related coefficients */
        self.calib.par_p1 = concat_bytes(
            coeff_array[BME68X_IDX_P1_MSB],
            coeff_array[BME68X_IDX_P1_LSB],
        );
        self.calib.par_p2 = (concat_bytes(
            coeff_array[BME68X_IDX_P2_MSB],
            coeff_array[BME68X_IDX_P2_LSB],
        )) as i16;
        self.calib.par_p3 = coeff_array[BME68X_IDX_P3] as i8;
        self.calib.par_p4 = (concat_bytes(
            coeff_array[BME68X_IDX_P4_MSB],
            coeff_array[BME68X_IDX_P4_LSB],
        )) as i16;
        self.calib.par_p5 = (concat_bytes(
            coeff_array[BME68X_IDX_P5_MSB],
            coeff_array[BME68X_IDX_P5_LSB],
        )) as i16;
        self.calib.par_p6 = (coeff_array[BME68X_IDX_P6]) as i8;
        self.calib.par_p7 = (coeff_array[BME68X_IDX_P7]) as i8;
        self.calib.par_p8 = (concat_bytes(
            coeff_array[BME68X_IDX_P8_MSB],
            coeff_array[BME68X_IDX_P8_LSB],
        )) as i16;
        self.calib.par_p9 = (concat_bytes(
            coeff_array[BME68X_IDX_P9_MSB],
            coeff_array[BME68X_IDX_P9_LSB],
        )) as i16;
        self.calib.par_p10 = (coeff_array[BME68X_IDX_P10]);

        /* Humidity related coefficients */
        self.calib.par_h1 = (((coeff_array[BME68X_IDX_H1_MSB] as u16) << 4)
            | ((coeff_array[BME68X_IDX_H1_LSB] as u16) & BME68X_BIT_H1_DATA_MSK));
        self.calib.par_h2 = (((coeff_array[BME68X_IDX_H2_MSB] as u16) << 4)
            | (((coeff_array[BME68X_IDX_H2_LSB]) as u16) >> 4));
        self.calib.par_h3 = coeff_array[BME68X_IDX_H3] as i8;
        self.calib.par_h4 = coeff_array[BME68X_IDX_H4] as i8;
        self.calib.par_h5 = coeff_array[BME68X_IDX_H5] as i8;
        self.calib.par_h6 = coeff_array[BME68X_IDX_H6];
        self.calib.par_h7 = coeff_array[BME68X_IDX_H7] as i8;

        /* Gas heater related coefficients */
        self.calib.par_gh1 = coeff_array[BME68X_IDX_GH1] as i8;
        self.calib.par_gh2 = (concat_bytes(
            coeff_array[BME68X_IDX_GH2_MSB],
            coeff_array[BME68X_IDX_GH2_LSB],
        )) as i16;
        self.calib.par_gh3 = coeff_array[BME68X_IDX_GH3] as i8;

        /* Other coefficients */
        self.calib.res_heat_range =
            ((coeff_array[BME68X_IDX_RES_HEAT_RANGE] & BME68X_RHRANGE_MSK) / 16);
        self.calib.res_heat_val = coeff_array[BME68X_IDX_RES_HEAT_VAL] as i8;
        self.calib.range_sw_err =
            ((coeff_array[BME68X_IDX_RANGE_SW_ERR] & BME68X_RSERROR_MSK) as i8) / 16;

        Ok(())
    }

    /// Read the variant ID information register status
    ///
    /// # Errors
    /// Errors if reading the register failed
    fn read_variant_id(&mut self) -> Result<(), BME68xError> {
        let data = self.get_regs(BME68xRegister::VariantId, 1)?;
        self.variant_id = u32::from(data[0]);
        Ok(())
    }

    /// Calcualte the temperature as a float
    ///
    /// # Arguments
    /// * `temp_adc`: The raw ADC Temperature
    ///
    /// Returns
    /// The temperature as a float
    fn calc_temperature(&mut self, temp_adc: u32) -> f32 {
        let par_t1_f32 = self.calib.par_t1 as f32;
        let par_t3_f32 = self.calib.par_t3 as f32;
        let temp_f32 = temp_adc as f32;

        let var1 = ((temp_f32 / 16384.0) - (par_t1_f32 / 1024.0)) * (par_t1_f32);

        let var2 = (((temp_f32 / 131072.0) - (par_t1_f32 / 8192.0))
            * ((temp_f32 / 131072.0) - (par_t1_f32 / 8192.0)))
            * (par_t3_f32 * 16.0);

        self.calib.t_fine = var1 + var2;

        self.calib.t_fine / 5120.0
    }

    /// Calcualte the pressure value as a float
    ///
    /// # Arguments
    /// * `pres_adc`: Raw pressure ADC value
    ///
    /// # Returns
    /// Pressure value as a float
    fn calc_pressure(&self, pres_adc: u32) -> f32 {
        let var1 = (self.calib.t_fine / 2.0) - 64000.0;
        let var2 = var1 * var1 * (self.calib.par_p6 as f32 / 131072.0);
        let var2 = var2 + (var1 * (self.calib.par_p5 as f32 * 2.0));
        let var2 = (var2 / 4.0) + ((self.calib.par_p4 as f32) * 65536.0);
        let var1 = (((self.calib.par_p3 as f32 * var1 * var1) / 16384.0)
            + (self.calib.par_p2 as f32 * var1))
            / 524288.0;
        let var1 = (1.0 + (var1 / 32768.0)) * (self.calib.par_p1 as f32);
        let calc_pres = 1048576.0 - (pres_adc as f32);

        if var1 != 0.0 {
            let calc_pres = ((calc_pres - (var2 / 4096.0)) * 6250.0) / var1;
            let var1 = ((self.calib.par_p9 as f32) * calc_pres * calc_pres) / 2147483648.0;
            let var2 = calc_pres * ((self.calib.par_p8 as f32) / 32768.0);
            let var3 = (calc_pres / 256.0)
                * (calc_pres / 256.0)
                * (calc_pres / 256.0)
                * (self.calib.par_p10 as f32 / 131072.0);
            calc_pres + (var1 + var2 + var3 + (self.calib.par_p7 as f32 * 128.0)) / 16.0
        } else {
            0.0
        }
    }

    /// Calcualte the humidity value as a float
    ///
    /// # Arguments
    /// * `hum_adc`: Raw humidty ADC value
    fn calc_humidity(&self, hum_adc: u32) -> f32 {
        let temp_comp = (self.calib.t_fine) / 5120.0;
        let var1 = (hum_adc as f32)
            - ((self.calib.par_h1 as f32 * 16.0) + ((self.calib.par_h3 as f32 / 2.0) * temp_comp));
        let var2 = var1
            * ((self.calib.par_h2 as f32 / 262144.0)
                * (1.0
                    + ((self.calib.par_h4 as f32 / 16384.0) * temp_comp)
                    + ((self.calib.par_h5 as f32 / 1048576.0) * temp_comp * temp_comp)));
        let var3 = self.calib.par_h6 as f32 / 16384.0;
        let var4 = self.calib.par_h7 as f32 / 2097152.0;
        let calc_hum = var2 + ((var3 + (var4 * temp_comp)) * var2 * var2);

        if calc_hum > 100.0 {
            100.0
        } else if calc_hum < 0.0 {
            0.0
        } else {
            calc_hum
        }
    }

    /// Calculate gas resistance high value as a float
    ///
    /// # Arguments:
    /// * `gas_res_adc`: Raw ADC gas resistance value
    /// * `gas_range`: The gas range to use for the calculation
    // TODO: Move outside struct?
    fn calc_gas_resistance_high(gas_res_adc: u16, gas_range: u8) -> f32 {
        let var1: u32 = 262144 >> gas_range;
        let var2: i32 = (gas_res_adc as i32) - 512;
        let var2 = var2 * 3;
        let var2 = 4096 + var2;
        1000000.0 * var1 as f32 / var2 as f32
    }

    /// Calculate gas resistance low value as a float
    ///
    /// # Arguments:
    /// * `gas_res_adc`: Raw ADC gas resistance value
    /// * `gas_range`: The gas range to use for the calculation
    fn calc_gas_resistance_low(&self, gas_res_adc: u16, gas_range: u8) -> f32 {
        let gas_res_f = gas_res_adc as f32;
        let gas_range_f = (1 << gas_range) as f32;
        let lookup_k1_range = [
            0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -0.8, 0.0, 0.0, -0.2, -0.5, 0.0, -1.0, 0.0, 0.0,
        ];
        let lookup_k2_range = [
            0.0, 0.0, 0.0, 0.0, 0.1, 0.7, 0.0, -0.8, -0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];

        let var1 = 1340.0 + (5.0 * self.calib.range_sw_err as f32);
        let var2 = (var1) * (1.0 + lookup_k1_range[gas_range as usize] / 100.0);
        let var3 = 1.0 + (lookup_k2_range[gas_range as usize] / 100.0);

        1.0 / (var3 * (0.000000125) * gas_range_f * (((gas_res_f - 512.0) / var2) + 1.0))
    }

    /// Calculate the heater resistance to a float
    ///
    /// # Arguments
    ///  * `temp`: The temperature
    fn calc_res_heat(&self, temp: u16) -> u8 {
        let temp = if temp > 400 { 400.0 } else { temp as f32 };

        let var1 = (self.calib.par_gh1 as f32 / (16.0)) + 49.0;
        let var2 = ((self.calib.par_gh2 as f32 / (32768.0)) * (0.0005)) + 0.00235;
        let var3 = self.calib.par_gh3 as f32 / (1024.0);
        let var4 = var1 * (1.0 + (var2 * temp));
        let var5 = var4 + (var3 * self.amb_temp as f32);
        (3.4 * ((var5
            * (4.0 / (4.0 + self.calib.res_heat_range as f32))
            * (1.0 / (1.0 + (self.calib.res_heat_val as f32 * 0.002))))
            - 25.0)) as u8
    }

    /// Read a single data from teh senssor
    fn read_field_data(&self, index: u8, data: &mut [BME68xData]) -> Result<(), BME68xError> {
        todo!()
    }

    /// Read all data fields of the sensor
    fn read_all_field_data(&self, data: &mut [BME68xData; 3]) -> Result<(), BME68xError> {
        todo!()
    }

    /// Switch between SPI memory pages
    fn set_mem_page(&self, reg_addr: u8) -> Result<(), BME68xError> {
        todo!()
    }

    /// Get The current SPI memory page
    fn get_mem_page(&self) -> Result<(), BME68xError> {
        todo!()
    }

    /// Set heater configuration
    fn set_conf(&self, conf: &BME68xHeatrConf, op_mode: BME68xOpMode) -> Result<u8, BME68xError> {
        let mut nb_conv = 0;
        let mut write_len = 0;
        let mut rh_reg_addr = [0; 10];
        let mut rh_reg_data = [0; 10];
        let mut gw_reg_addr = [0; 10];
        let mut gw_reg_data = [0; 10];

        match op_mode {
            BME68xOpMode::ForcedMode => {
                rh_reg_addr[0] = BME68xRegister::ResHeat0 as u8;
                rh_reg_data[0] = self.calc_res_heat(conf.heatr_temp);
                gw_reg_addr[0] = BME68xRegister::GasWait0 as u8;
                gw_reg_data[0] = calc_gas_wait(conf.heatr_dur);
                nb_conv = 0;
                write_len = 1;
            }
            BME68xOpMode::SequentialMode => {
                for i in 0..conf.profile_len {
                    let index = i as usize;
                    rh_reg_addr[index] = BME68xRegister::ResHeat0 as u8 + i;
                    rh_reg_data[index] = self.calc_res_heat(conf.heatr_temp_prof[index]);
                    gw_reg_addr[index] = BME68xRegister::GasWait0 as u8 + i;
                    gw_reg_data[index] = calc_gas_wait(conf.heatr_dur_prof[index]);
                    nb_conv = conf.profile_len;
                    write_len = conf.profile_len;
                }
            }
            BME68xOpMode::ParallelMode => {
                if conf.shared_heatr_dur == 0 {
                    return Err(BME68xError::DefineShdHeatrDur);
                }

                for i in 0..conf.profile_len {
                    let index = i as usize;
                    rh_reg_addr[index] = BME68xRegister::ResHeat0 as u8 + i;
                    rh_reg_data[index] = self.calc_res_heat(conf.heatr_temp_prof[index]);
                    gw_reg_addr[index] = BME68xRegister::GasWait0 as u8 + i;
                    gw_reg_data[index] = conf.heatr_dur_prof[index] as u8;
                    nb_conv = conf.profile_len;
                    write_len = conf.profile_len;
                    let shared_dur = calc_heatr_dur_shared(conf.shared_heatr_dur);
                    self.set_regs(&[BME68xRegister::ShdHeatrDur as u8], &[shared_dur], 1)?;
                }
            }
            _ => return Err(BME68xError::DefineOpMode),
        }

        self.set_regs(&rh_reg_addr, &rh_reg_data, write_len.into())?;
        self.set_regs(&gw_reg_addr, &gw_reg_data, write_len.into())?;

        Ok(nb_conv)
    }
}

/// Caclulate register value for shared heater duration
fn calc_heatr_dur_shared(mut dur: u16) -> u8 {
    let mut factor = 0;
    let heatdurval;
    if dur >= 0x783 {
        heatdurval = 0xff; // Max Duration
    } else {
        dur = ((u32::from(dur) * 1000) / 477) as u16;
        while dur > 0x3F {
            dur = dur >> 2;
            factor += 1;
        }
        heatdurval = (dur + (factor * 64)) as u8;
    }

    heatdurval
}

// TODO: Document properly
fn calc_gas_wait(mut dur: u16) -> u8 {
    let mut factor = 0;
    if dur >= 0xfc0 {
        0xff // Max Duration;
    } else {
        while dur > 0x3f {
            dur = dur / 4;
            factor += 1;
        }
        (dur + (factor * 64)) as u8
    }
}

/// Swap the contents of two fields.
///
/// # Arguments
/// * `index1`: Index of the first of the fields to swap.
/// * `index2`: Index of the second of the fields to swap.
/// * `field`: Mutable array of the fields to swap.
fn swap_fields(index1: usize, index2: usize, field: &mut [BME68xData]) {
    let temp = field[index1];
    field[index1] = field[index2];
    field[index2] = temp;
}

/// Sort the sensor data
fn sort_sensor_data(low_index: usize, high_index: usize, field: &mut [BME68xData]) {
    let meas_index1 = i16::from(field[low_index].meas_index);
    let meas_index2 = i16::from(field[high_index].meas_index);
    if ((field[low_index].status & BME68X_NEW_DATA_MSK) != 0)
        && ((field[high_index].status & BME68X_NEW_DATA_MSK) != 0)
    {
        let diff = meas_index2 - meas_index1;
        if ((diff > -3) && (diff < 0)) || (diff > 2) {
            swap_fields(low_index, high_index, field);
        }
    } else if (field[high_index].status & BME68X_NEW_DATA_MSK) != 0 {
        swap_fields(low_index, high_index, field);
    }
}

/// Concatenate two u8 into a u16
///
/// # Arguments
/// * `msb`: The most significant bit
/// * `lsb`: The least significant bit
// FIXME: Convert this to a macro.
fn concat_bytes(msb: u8, lsb: u8) -> u16 {
    (u16::from(msb) << 8) | u16::from(lsb)
}

/// Set bits for a register
// FIXME: Convert this to a macro
fn set_bits(reg_data: u8, bitmask: u8, bitpos: u8, data: u8) -> u8 {
    ((reg_data & !(bitmask)) | ((data << bitpos) & bitmask))
}

fn set_bits_pos_0(reg_data: u8, bitmask: u8, data: u8) -> u8 {
    ((reg_data & !(bitmask)) | (data & bitmask))
}

/// Get bits starting from positon 0
// FIXME: Convert to macro
fn get_bits(reg_data: u8, bitmask: u8, bitpos: u8) -> u8 {
    (reg_data & bitmask) >> bitpos
}

/// Analyze the sensor data
///
/// # Arguments
/// * `data`: Array of measurement data
/// * `n_meas`: Number of measurements
///
/// # Errors
/// Returns an error if analysis fails
fn analyze_sensor_data(data: &[BME68xData], n_meas: usize) -> Result<(), BME68xError> {
    if (data[0].temperature < BME68X_MIN_TEMPERATURE)
        || (data[0].temperature > BME68X_MAX_TEMPERATURE)
    {
        return Err(BME68xError::SelfTest);
    }

    if (data[0].pressure < BME68X_MIN_PRESSURE) || (data[0].pressure > BME68X_MAX_PRESSURE) {
        return Err(BME68xError::SelfTest);
    }

    if (data[0].humidity < BME68X_MIN_HUMIDITY) || (data[0].humidity > BME68X_MAX_HUMIDITY) {
        return Err(BME68xError::SelfTest);
    }
    for i in 0..n_meas {
        if (data[i].status & BME68X_GASM_VALID_MSK) == 0 {
            return Err(BME68xError::SelfTest);
        }
    }
    if n_meas >= 6 {
        let cent_res = ((5.0 * (data[3].gas_resistance + data[5].gas_resistance))
            / (2.0 * data[4].gas_resistance)) as u32;
        if cent_res < 6 {
            return Err(BME68xError::SelfTest);
        }
    }
    Ok(())
}
