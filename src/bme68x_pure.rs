//! BME68X Driver Implementation in pure rust.
// TODO: Conditional FPU support?

// FIXME: Get this moved into cargo.toml. IDK why it is nt working there
#![allow(clippy::unreadable_literal)]
use std::num::TryFromIntError;

// TODO: More enumerations to replace constants
use embedded_hal::i2c::I2c;

// Other stuff

///  Chip Unique Identifier
const BME68X_CHIP_ID: u8 = 0x61;

/// Soft Reset Command
const BME68X_SOFT_RESET_CMD: u8 = 0xb6;

/// Wait period for a soft reset
const BME68X_PERIOD_RESET: u32 = 10000;

/// Period between two polls
const BME68X_PERIOD_POLL: u32 = 10000;

// For self test
/// Self test heater duration 1
const BME68X_HEATR_DUR1: u16 = 1000;

/// Self Test Heater duration 2
const BME68X_HEATR_DUR2: u16 = 2000;

/// Self test low temperature
const BME68X_LOW_TEMP: u16 = 150;

/// Self Test High Temperature
const BME68X_HIGH_TEMP: u16 = 350;

/// Self Test Number of Measurements
const BME68X_N_MEAS: usize = 6;

/// Self Test Heater duration 1 delay time
const BME68X_HEATR_DUR1_DELAY: u32 = 1000000;

/// Self test heater duration 2 delay time
const BME68X_HEATR_DUR2_DELAY: u32 = 2000000;

// TODO: Make these an enum
/// Enable Heater
const BME68X_DISABLE_HEATER: u8 = 0x01;
/// Disable Heater
const BME68X_ENABLE_HEATER: u8 = 0x00;

/// Gas Measurement Enable Enum
enum BME68xGasEnable {
    /// Disable gas measurement
    Disable,

    /// Enable High Gas Measurement
    EnableHigh,

    /// Enable Low Gas Measurement
    EnableLow,
}

impl From<BME68xGasEnable> for u8 {
    fn from(value: BME68xGasEnable) -> Self {
        match value {
            BME68xGasEnable::Disable => 0x00,
            BME68xGasEnable::EnableLow => 0x01,
            BME68xGasEnable::EnableHigh => 0x02,
        }
    }
}

/// Enumeration of the Gas Sensing Variants
// TODO: Find out what exactly this is
enum BME68xVariant {
    /// Low gas variant
    GasLow,

    /// High Gas Variant
    GasHigh,
}

impl From<u8> for BME68xVariant {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::GasLow,
            0x01 => Self::GasHigh,
            _ => panic!("Cannot convert {value} to BME68xVariant"),
        }
    }
}

// Min/max values allowed for testing
/// Min temp of 0c
const BME68X_MIN_TEMPERATURE: f32 = 0.0;

/// Max Temp of 60c
const BME68X_MAX_TEMPERATURE: f32 = 60.0;

/// Min pressure of 900 Hecto Pascals
const BME68X_MIN_PRESSURE: f32 = 90000.0;

/// Max pressure of 1100 hecto pascals
const BME68X_MAX_PRESSURE: f32 = 110000.0;

/// Min humidity of 20%
const BME68X_MIN_HUMIDITY: f32 = 20.0;

/// Max humidity of 80%
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
const BME68X_LEN_FIELD: usize = 17;

///  Length between two fields
const BME68X_LEN_FIELD_OFFSET: u8 = 17;

///  Length of the configuration register
const BME68X_LEN_CONFIG: usize = 5;

///  Length of the interleaved buffer
const BME68X_LEN_INTERLEAVE_BUFF: usize = 20;

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

/// Enumeration of the device addresses
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum BME68xAddr {
    /// Low Address
    LOW = 0x76,

    /// High Address
    HIGH = 0x77,
}

/// Enumeration of the memory pages for SPI mode
#[repr(u8)]
#[derive(Clone, Copy)]
enum BME68xMemPage {
    /// SPI Memory Page 0
    Page0 = 0x10,

    /// SPI Memory Page 1
    Page1 = 0x00,
}

impl From<u8> for BME68xMemPage {
    fn from(value: u8) -> Self {
        match value {
            0x10 => Self::Page0,
            0x00 => Self::Page1,
            _ => panic!("Cannot convert {value} to a BME68xMemPage"),
        }
    }
}

impl From<BME68xMemPage> for u8 {
    fn from(value: BME68xMemPage) -> Self {
        value as u8
    }
}

/// Chip Error codes
// #[repr(i8)]
#[derive(Debug, Clone, Copy)]
pub enum BME68xError {
    /// Success
    Ok,

    /// Null Pointer Passed
    NullPtr,

    /// Communication Failure
    ComFail,

    /// Sensor Not Found
    DevNotFound,

    /// Incorrect Length Parameter
    InvalidLength,

    /// Self Test Error
    SelfTest,

    /// Casting Error. Should never happen, but putting it here so we
    /// have a recoverable error if it magically does happen
    CastError,

    // These are warnings, Can technically proceed if this occur
    /// Define a valid operation mode
    DefineOpMode,

    /// No New Data was found
    NoNewData,

    /// Define shared heating duration
    DefineShdHeatrDur,
}

impl From<TryFromIntError> for BME68xError {
    fn from(_value: TryFromIntError) -> Self {
        Self::CastError
    }
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

impl From<BME68xRegister> for u8 {
    fn from(value: BME68xRegister) -> Self {
        value as u8
    }
}

/// BME68X Oversampling Settings
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
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

impl BME68xOs {
    /// Get the number of measurement cycles for each oversampling option.
    ///
    /// # Returns
    /// The number of measurement cycles for the given oversampling option.
    fn get_meas_cycles(self) -> u32 {
        match self {
            BME68xOs::OsNone => 0,
            BME68xOs::Os1x => 1,
            BME68xOs::Os2x => 2,
            BME68xOs::Os4x => 4,
            BME68xOs::Os8x => 8,
            BME68xOs::Os16x => 16,
        }
    }
}

impl From<BME68xOs> for u8 {
    fn from(value: BME68xOs) -> Self {
        value as u8
    }
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

impl From<BME68xODR> for u8 {
    fn from(value: BME68xODR) -> Self {
        value as u8
    }
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

/// Enumertion of possible filter options
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum BME68xFilter {
    /// No Filtering
    Off = 0,

    /// Filter Coefficent of 2
    Size1 = 1,

    /// Filter Coefficcent of 4,
    Size3 = 2,

    /// Filter Coefficent of 8
    Size7 = 3,

    /// Filter Coefficent of 16
    Size15 = 4,

    /// Filter Coefficent of 32,
    Size31 = 5,

    /// Filter Coefficent of 64
    Size63 = 6,

    /// Filter Coefficent of 128,
    Size127 = 7,
}

impl From<BME68xFilter> for u8 {
    fn from(value: BME68xFilter) -> Self {
        value as u8
    }
}

impl From<u8> for BME68xFilter {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::Size1,
            2 => Self::Size3,
            3 => Self::Size7,
            4 => Self::Size15,
            5 => Self::Size31,
            6 => Self::Size63,
            7 => Self::Size127,
            _ => panic!("Cannot convert {value} into BME68xFilter"),
        }
    }
}

/// Enumeration of possible interfaces for the sensor.
#[derive(Clone, Copy)]
pub enum BME68xIntf {
    /// SPI Interface
    SPIIntf,

    /// I2C Interface
    I2CIntf,
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

impl From<BME68xOpMode> for u8 {
    fn from(value: BME68xOpMode) -> Self {
        value as u8
    }
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
#[derive(Debug, Clone, Copy, Default)]
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
    #[must_use]
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
struct BME68xCalibData {
    /// Calibration coefficient for the humidity sensor
    par_h1: u16,

    /// Calibration coefficient for the humidity sensor
    par_h2: u16,

    /// Calibration coefficient for the humidity sensor
    par_h3: i8,

    /// Calibration coefficient for the humidity sensor
    par_h4: i8,

    /// Calibration coefficient for the humidity sensor
    par_h5: i8,

    /// Calibration coefficient for the humidity sensor
    par_h6: u8,

    /// Calibration coefficient for the humidity sensor
    par_h7: i8,

    /// Calibration coefficient for the gas sensor
    par_gh1: i8,

    /// Calibration coefficient for the gas sensor
    par_gh2: i16,

    /// Calibration coefficient for the gas sensor
    par_gh3: i8,

    /// Calibration coefficient for the temperature sensor
    par_t1: u16,

    /// Calibration coefficient for the temperature sensor
    par_t2: i16,

    /// Calibration coefficient for the temperature sensor
    par_t3: i8,

    /// Calibration coefficient for the pressure sensor
    par_p1: u16,

    /// Calibration coefficient for the pressure sensor
    par_p2: i16,

    /// Calibration coefficient for the pressure sensor
    par_p3: i8,

    /// Calibration coefficient for the pressure sensor
    par_p4: i16,

    /// Calibration coefficient for the pressure sensor
    par_p5: i16,

    /// Calibration coefficient for the pressure sensor
    par_p6: i8,

    /// Calibration coefficient for the pressure sensor
    par_p7: i8,

    /// Calibration coefficient for the pressure sensor
    par_p8: i16,

    /// Calibration coefficient for the pressure sensor
    par_p9: i16,

    /// Calibration coefficient for the pressure sensor
    par_p10: u8,

    /// Variable to store the intermediate temperature coefficient
    t_fine: f32,

    /// Heater resistance range coefficient
    res_heat_range: u8,

    /// Heater resistance value coefficient
    res_heat_val: i8,

    /// Gas resistance range switching error coefficient
    range_sw_err: i8,
}

impl BME68xCalibData {
    /// Create an empty instance
    fn new() -> Self {
        Self {
            par_gh1: 0,
            par_h1: 0,
            par_h2: 0,
            par_h3: 0,
            par_h4: 0,
            par_h5: 0,
            par_h6: 0,
            par_h7: 0,
            par_gh2: 0,
            par_gh3: 0,
            par_t1: 0,
            par_t2: 0,
            par_p1: 0,
            par_t3: 0,
            par_p10: 0,
            par_p2: 0,
            par_p3: 0,
            par_p4: 0,
            par_p5: 0,
            par_p6: 0,
            par_p7: 0,
            par_p8: 0,
            par_p9: 0,
            range_sw_err: 0,
            res_heat_range: 0,
            res_heat_val: 0,
            t_fine: 0.0,
        }
    }
}

/// BME68X sensor settings structure which comprises of ODR, over-sampling and filter settings.
#[derive(Clone, Copy, Debug)]
pub struct BME68xConf {
    /// Humidity overrsampling
    pub os_hum: BME68xOs,

    /// Temperature Oversampling
    pub os_temp: BME68xOs,

    /// Pressure Oversampling
    pub os_pres: BME68xOs,

    /// Filter Coefficent
    pub filter: BME68xFilter,

    /// Standby time between sequential mode measurement profiles
    pub odr: BME68xODR,
}

/// Gas Heater Configuration
#[derive(Debug, Clone, Copy, Default)]
pub struct BME68xHeatrConf {
    /// Enable gas measurement
    pub enable: bool,

    /// Store the heater temperature for forced mode degree Celsius
    pub heatr_temp: u16,

    /// Store the heating duration for forced mode in milliseconds
    pub heatr_dur: u16,

    /// Store the heater temperature profile in degree Celsius, Max of 10
    pub heatr_temp_prof: [u16; 10],

    /// Store the heating duration profile in milliseconds. Max of 10
    pub heatr_dur_prof: [u16; 10],

    /// Variable to store the length of the heating profile
    pub profile_len: u8,

    /// Variable to store heating duration for parallel mode in milliseconds
    pub shared_heatr_dur: u16,
}

// TODO: constructors for each mode
impl BME68xHeatrConf {
    /// Create a new empty instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            enable: false,
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
pub struct BME68xDev<I2C> {
    // FIXME: Instead need to support I2C or SPI
    /// Concrete I2C Implementation
    i2c: I2C,

    /// The I2C Address
    address: BME68xAddr,

    /// Chip ID
    chip_id: u8,

    /// Variant ID.
    variant_id: BME68xVariant,

    /// SPI/I2C Interface
    intf: BME68xIntf,

    /// Memory page used
    mem_page: BME68xMemPage,

    /// Ambient Temperature in degrees C
    amb_temp: i8,

    /// Sensor Calibration Data
    calib: BME68xCalibData,

    /// To store interface pointer error
    intf_rslt: BME68xError,

    /// Store info messages
    info_msg: BME68xError,

    /// Function to delay by a specific numebr of microseconds
    delay_us: Box<dyn Fn(u32)>,
}

impl From<BME68xAddr> for u8 {
    fn from(value: BME68xAddr) -> Self {
        value as u8
    }
}

impl<I2C: I2c> BME68xDev<I2C> {
    /// Create a new instance of the sensor
    ///
    /// # Arguments
    /// * `bus`: The communication bus to use for talking with the sensor
    /// * `address`: The address to use for talking to the sensor
    /// * `amb_temp`: Ambient temperature to use for compensation, in degrees C. 25 is a safe value for this
    /// * `intf`: The interface type to use for communication
    /// * `delay_us`: Function to use for performing delay in microseconds.
    pub fn new(
        bus: I2C,
        address: BME68xAddr,
        amb_temp: i8,
        intf: BME68xIntf,
        delay_us: Box<dyn Fn(u32)>,
    ) -> Self {
        Self {
            address,
            i2c: bus,
            chip_id: 0,
            amb_temp,
            variant_id: BME68xVariant::GasLow,
            intf,
            mem_page: BME68xMemPage::Page0,
            calib: BME68xCalibData::new(),
            intf_rslt: BME68xError::Ok,
            info_msg: BME68xError::Ok,
            delay_us,
        }
    }

    /// Initialize the sensor.
    ///
    /// Reads the Chip ID and calibrates the sensor.
    /// This should be called before all other functions.
    ///
    /// # Errors
    /// Returns an error if the initialzation is unsuccessful.
    pub fn init(&mut self) -> Result<(), BME68xError> {
        self.soft_reset()?;

        let mut data = [0; 1];
        self.get_regs(BME68xRegister::ChipId.into(), &mut data)?;
        self.chip_id = data[0];
        if self.chip_id == BME68X_CHIP_ID {
            self.read_variant_id()?;
            self.get_calib_data()
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
    pub fn set_regs(
        &mut self,
        reg_addr: &[u8],
        reg_data: &[u8],
        len: usize,
    ) -> Result<(), BME68xError> {
        // FIXME: Proper spi support
        let mut tmp_buff = [0; BME68X_LEN_INTERLEAVE_BUFF];
        if (len > 0) && (len <= (BME68X_LEN_INTERLEAVE_BUFF / 2)) {
            for index in 0..len {
                if matches!(self.intf, BME68xIntf::SPIIntf) {
                    self.set_mem_page(reg_addr[index])?;
                    tmp_buff[2 * index] = reg_addr[index] & BME68X_SPI_WR_MSK;
                } else {
                    tmp_buff[2 * index] = reg_addr[index];
                }
                tmp_buff[(2 * index) + 1] = reg_data[index];
            }
            let result = self.i2c.write(self.address.into(), &tmp_buff[0..(2 * len)]);
            if result.is_ok() {
                self.intf_rslt = BME68xError::Ok;
                Ok(())
            } else {
                self.intf_rslt = BME68xError::ComFail;
                Err(BME68xError::ComFail)
            }
        } else {
            Err(BME68xError::InvalidLength)
        }
    }

    /// Read data from the given registers
    ///
    /// # Arguments
    /// * `reg_addr`: Register addresses to read data from
    /// * `data`: The buffer to place the read data in.
    ///
    /// # Returns
    /// Data read from the registers
    ///
    /// # Errors
    /// Errors if failing to read from the registers.
    // TODO: Remove the len argument?
    // FIXME: Second version for reading a single register that does not need the "data" buf
    pub fn get_regs(&mut self, mut reg_addr: u8, data: &mut [u8]) -> Result<(), BME68xError> {
        // FIXME: Proper SPI support
        if matches!(self.intf, BME68xIntf::SPIIntf) {
            self.set_mem_page(reg_addr)?;
            reg_addr |= BME68X_SPI_RD_MSK;
        }
        let result = self.i2c.write_read(self.address.into(), &[reg_addr], data);

        if result.is_ok() {
            self.intf_rslt = BME68xError::Ok;
            Ok(())
        } else {
            self.intf_rslt = BME68xError::ComFail;
            Err(BME68xError::ComFail)
        }
    }

    /// Soft-Reset the sensorr
    ///
    /// # Errors
    /// Returns an error if soft-resetting the sensor failed.
    pub fn soft_reset(&mut self) -> Result<(), BME68xError> {
        if matches!(self.intf, BME68xIntf::SPIIntf) {
            self.get_mem_page()?;
        }
        self.set_regs(
            &[BME68xRegister::SoftReset.into()],
            &[BME68X_SOFT_RESET_CMD],
            1,
        )?;
        (self.delay_us)(BME68X_PERIOD_RESET);
        if matches!(self.intf, BME68xIntf::SPIIntf) {
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
    pub fn set_op_mode(&mut self, op_mode: BME68xOpMode) -> Result<(), BME68xError> {
        let mut tmp_pow_mode;
        loop {
            let mut data = [0; 1];
            self.get_regs(BME68xRegister::CtrlMeas.into(), &mut data)?;
            tmp_pow_mode = data[0];
            let pow_mode: BME68xOpMode = (tmp_pow_mode & BME68X_MODE_MSK).into();

            if !matches!(pow_mode, BME68xOpMode::SleepMode) {
                // In rust ! is bitwise not
                tmp_pow_mode &= !BME68X_MODE_MSK; /* Set to sleep */
                self.set_regs(&[BME68xRegister::CtrlMeas.into()], &[tmp_pow_mode], 1)?;
                (self.delay_us)(BME68X_PERIOD_POLL);
            } else {
                break;
            }
        }
        /* Already in sleep */
        if !matches!(op_mode, BME68xOpMode::SleepMode) {
            tmp_pow_mode = (tmp_pow_mode & !BME68X_MODE_MSK) | (op_mode as u8 & BME68X_MODE_MSK);
            self.set_regs(&[BME68xRegister::CtrlMeas.into()], &[tmp_pow_mode], 1)?;
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
    pub fn get_op_mode(&mut self) -> Result<BME68xOpMode, BME68xError> {
        let mut data = [0; 1];
        self.get_regs(BME68xRegister::CtrlMeas.into(), &mut data)?;
        let output = data[0];
        Ok(BME68xOpMode::from(output & BME68X_MODE_MSK))
    }

    /// Get the remaining duration that can be used for heating
    ///
    /// # Arguments
    /// * `op_mode`: The operation mode of the sensor
    /// * `conf`: The sensor configuration.
    pub fn get_meas_dur(&self, op_mode: BME68xOpMode, conf: &BME68xConf) -> u32 {
        let mut meas_dur;

        let meas_cycles = conf.os_temp.get_meas_cycles()
            + conf.os_pres.get_meas_cycles()
            + conf.os_hum.get_meas_cycles();

        // TPH Measurement Duration
        meas_dur = meas_cycles * 1963;
        meas_dur += 477 * 4; // TPH Switching Duration
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
    /// Tuple wheter the first element is the sensor data, and the second is the
    /// number of read elements
    ///
    /// # Errors
    pub fn get_data(
        &mut self,
        op_mode: BME68xOpMode,
    ) -> Result<([BME68xData; 3], u8), BME68xError> {
        let mut new_fields = 0;
        let mut data = [BME68xData::new(); 3];
        match op_mode {
            BME68xOpMode::ForcedMode => {
                self.read_field_data(0, &mut data[0])?;
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
            BME68xOpMode::SleepMode => return Err(BME68xError::DefineOpMode),
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

        self.get_regs(BME68xRegister::CtrlGas1.into(), &mut data_array)?;
        self.info_msg = BME68xError::Ok;
        data_array[4] = set_bits(
            data_array[4],
            BME68X_FILTER_MSK,
            BME68X_FILTER_POS,
            conf.filter.into(),
        );
        data_array[3] = set_bits(
            data_array[3],
            BME68X_OST_MSK,
            BME68X_OST_POS,
            conf.os_temp.into(),
        );
        data_array[3] = set_bits(
            data_array[3],
            BME68X_OSP_MSK,
            BME68X_OSP_POS,
            conf.os_pres.into(),
        );
        data_array[1] = set_bits_pos_0(data_array[1], BME68X_OSH_MSK, conf.os_hum.into());
        if !matches!(conf.odr, BME68xODR::ODRNone) {
            odr20 = conf.odr.into();
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
    pub fn get_config(&mut self) -> Result<BME68xConf, BME68xError> {
        let mut data_array = [0; 5];
        self.get_regs(BME68xRegister::CtrlGas1.into(), &mut data_array)?;
        Ok(BME68xConf {
            os_hum: BME68xOs::from(data_array[1] & BME68X_OSH_MSK),
            filter: BME68xFilter::from(get_bits(
                data_array[4],
                BME68X_FILTER_MSK,
                BME68X_FILTER_POS,
            )),
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
        &mut self,
        op_mode: BME68xOpMode,
        conf: &BME68xHeatrConf,
    ) -> Result<(), BME68xError> {
        self.set_op_mode(BME68xOpMode::SleepMode)?;
        let hctrl;
        let run_gas;
        let mut ctrl_gas_data = [0; 2];
        let ctrl_gas_addr = [
            BME68xRegister::CtrlGas0.into(),
            BME68xRegister::CtrlGas1.into(),
        ];

        let nb_conv = self.set_conf(conf, op_mode)?;
        self.get_regs(BME68xRegister::CtrlGas0.into(), &mut ctrl_gas_data)?;

        if conf.enable {
            hctrl = BME68X_ENABLE_HEATER;
            if matches!(self.variant_id, BME68xVariant::GasHigh) {
                run_gas = BME68xGasEnable::EnableHigh;
            } else {
                run_gas = BME68xGasEnable::EnableLow;
            }
        } else {
            hctrl = BME68X_DISABLE_HEATER;
            run_gas = BME68xGasEnable::Disable;
        }

        ctrl_gas_data[0] = set_bits(ctrl_gas_data[0], BME68X_HCTRL_MSK, BME68X_HCTRL_POS, hctrl);
        ctrl_gas_data[1] = set_bits_pos_0(ctrl_gas_data[1], BME68X_NBCONV_MSK, nb_conv);
        ctrl_gas_data[1] = set_bits(
            ctrl_gas_data[1],
            BME68X_RUN_GAS_MSK,
            BME68X_RUN_GAS_POS,
            run_gas.into(),
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
    pub fn get_heatr_conf(&mut self) -> Result<BME68xHeatrConf, BME68xError> {
        let mut conf = BME68xHeatrConf::new();
        /* FIXME: Add conversion to deg C and ms and add the other parameters. This is copied from the original BME68x.c file  */

        let mut data = [0; 10];

        // FIXME: Pass in profile len conf, like in the original API.
        self.get_regs(BME68xRegister::ResHeat0.into(), &mut data)?;
        conf.heatr_temp_prof = data.map(std::convert::Into::into);

        // FIXME: Pass in profile len conf, like in the original API.
        self.get_regs(BME68xRegister::GasWait0.into(), &mut data)?;
        conf.heatr_dur_prof = data.map(std::convert::Into::into);

        Ok(conf)
    }

    /// Perform a self test of the low gas variant of the sensor
    ///
    /// # Errors
    /// Returns an error if the self test failed.
    // FIXME This function is not working right now. Not sure why
    pub fn selftest_check(&mut self) -> Result<(), BME68xError> {
        // TODO: Figure out how we can re-enable cloning for test?
        // let mut t_dev = self.clone();
        let conf = BME68xConf {
            os_hum: BME68xOs::Os1x,
            os_pres: BME68xOs::Os16x,
            os_temp: BME68xOs::Os2x,
            filter: BME68xFilter::Off,
            odr: BME68xODR::from(0),
        };
        let mut heatr_conf = BME68xHeatrConf {
            enable: true,
            heatr_dur: BME68X_HEATR_DUR1,
            heatr_temp: BME68X_HIGH_TEMP,
            heatr_dur_prof: [0; 10],
            heatr_temp_prof: [0; 10],
            profile_len: 0,
            shared_heatr_dur: 0,
        };
        self.init()?;
        self.set_heatr_conf(BME68xOpMode::ForcedMode, &heatr_conf)?;
        self.set_config(&conf)?;
        self.set_op_mode(BME68xOpMode::ForcedMode)?;

        // Wait for measurement to complete
        (self.delay_us)(BME68X_HEATR_DUR1_DELAY);
        let (data, _) = self.get_data(BME68xOpMode::ForcedMode)?;

        if (data[0].idac != 0x00)
            && (data[0].idac != 0xFF)
            && ((data[0].status & BME68X_GASM_VALID_MSK) != 0)
        {
            // Do Nothiung
        } else {
            return Err(BME68xError::SelfTest);
        }

        heatr_conf.heatr_dur = BME68X_HEATR_DUR2;

        let mut data = [BME68xData::new(); BME68X_N_MEAS];
        let mut i = 0;
        while i < BME68X_N_MEAS {
            if (i % 2) == 0 {
                heatr_conf.heatr_temp = BME68X_HIGH_TEMP;
            } else {
                heatr_conf.heatr_temp = BME68X_LOW_TEMP;
            }
            self.set_heatr_conf(BME68xOpMode::ForcedMode, &heatr_conf)?;
            self.set_config(&conf)?;
            self.set_op_mode(BME68xOpMode::ForcedMode)?;

            // Wait for measurement to complete
            (self.delay_us)(BME68X_HEATR_DUR2_DELAY);
            let (samples, _) = self.get_data(BME68xOpMode::ForcedMode)?;
            data[i] = samples[0];
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
        self.get_regs(
            BME68xRegister::Coeff1.into(),
            &mut coeff_array[0..BME68X_LEN_COEFF1],
        )?;

        // Read second chunk of coefficents.
        self.get_regs(
            BME68xRegister::Coeff2.into(),
            &mut coeff_array[BME68X_LEN_COEFF1..(BME68X_LEN_COEFF1 + BME68X_LEN_COEFF2)],
        )?;

        // Read the third chunk of coefficents
        self.get_regs(
            BME68xRegister::Coeff3.into(),
            &mut coeff_array[(BME68X_LEN_COEFF1 + BME68X_LEN_COEFF2)
                ..(BME68X_LEN_COEFF1 + BME68X_LEN_COEFF2 + BME68X_LEN_COEFF3)],
        )?;

        // Copy data over
        /* Temperature related coefficients */
        // FIXME: The unsigned to signed conversions might not properly wrap
        self.calib.par_t1 = concat_bytes(
            coeff_array[BME68X_IDX_T1_MSB],
            coeff_array[BME68X_IDX_T1_LSB],
        );
        self.calib.par_t2 = wrap_u2i16(concat_bytes(
            coeff_array[BME68X_IDX_T2_MSB],
            coeff_array[BME68X_IDX_T2_LSB],
        ));
        self.calib.par_t3 = wrap_u2i8(coeff_array[BME68X_IDX_T3]);

        /* Pressure related coefficients */
        self.calib.par_p1 = concat_bytes(
            coeff_array[BME68X_IDX_P1_MSB],
            coeff_array[BME68X_IDX_P1_LSB],
        );
        self.calib.par_p2 = wrap_u2i16(concat_bytes(
            coeff_array[BME68X_IDX_P2_MSB],
            coeff_array[BME68X_IDX_P2_LSB],
        ));
        self.calib.par_p3 = wrap_u2i8(coeff_array[BME68X_IDX_P3]);
        self.calib.par_p4 = wrap_u2i16(concat_bytes(
            coeff_array[BME68X_IDX_P4_MSB],
            coeff_array[BME68X_IDX_P4_LSB],
        ));
        self.calib.par_p5 = wrap_u2i16(concat_bytes(
            coeff_array[BME68X_IDX_P5_MSB],
            coeff_array[BME68X_IDX_P5_LSB],
        ));
        self.calib.par_p6 = wrap_u2i8(coeff_array[BME68X_IDX_P6]);
        self.calib.par_p7 = wrap_u2i8(coeff_array[BME68X_IDX_P7]);
        self.calib.par_p8 = wrap_u2i16(concat_bytes(
            coeff_array[BME68X_IDX_P8_MSB],
            coeff_array[BME68X_IDX_P8_LSB],
        ));
        self.calib.par_p9 = wrap_u2i16(concat_bytes(
            coeff_array[BME68X_IDX_P9_MSB],
            coeff_array[BME68X_IDX_P9_LSB],
        ));
        self.calib.par_p10 = coeff_array[BME68X_IDX_P10];

        /* Humidity related coefficients */
        self.calib.par_h1 = (u16::from(coeff_array[BME68X_IDX_H1_MSB]) << 4)
            | (u16::from(coeff_array[BME68X_IDX_H1_LSB]) & BME68X_BIT_H1_DATA_MSK);
        self.calib.par_h2 = (u16::from(coeff_array[BME68X_IDX_H2_MSB]) << 4)
            | (u16::from(coeff_array[BME68X_IDX_H2_LSB]) >> 4);
        self.calib.par_h3 = wrap_u2i8(coeff_array[BME68X_IDX_H3]);
        self.calib.par_h4 = wrap_u2i8(coeff_array[BME68X_IDX_H4]);
        self.calib.par_h5 = wrap_u2i8(coeff_array[BME68X_IDX_H5]);
        self.calib.par_h6 = coeff_array[BME68X_IDX_H6];
        self.calib.par_h7 = wrap_u2i8(coeff_array[BME68X_IDX_H7]);

        /* Gas heater related coefficients */
        self.calib.par_gh1 = wrap_u2i8(coeff_array[BME68X_IDX_GH1]);
        self.calib.par_gh2 = wrap_u2i16(concat_bytes(
            coeff_array[BME68X_IDX_GH2_MSB],
            coeff_array[BME68X_IDX_GH2_LSB],
        ));
        self.calib.par_gh3 = wrap_u2i8(coeff_array[BME68X_IDX_GH3]);

        /* Other coefficients */
        self.calib.res_heat_range =
            (coeff_array[BME68X_IDX_RES_HEAT_RANGE] & BME68X_RHRANGE_MSK) / 16;
        self.calib.res_heat_val = wrap_u2i8(coeff_array[BME68X_IDX_RES_HEAT_VAL]);
        self.calib.range_sw_err =
            (wrap_u2i8(coeff_array[BME68X_IDX_RANGE_SW_ERR] & BME68X_RSERROR_MSK)) / 16;

        Ok(())
    }

    /// Read the variant ID information register status
    ///
    /// # Errors
    /// Errors if reading the register failed
    fn read_variant_id(&mut self) -> Result<(), BME68xError> {
        let mut data = [0; 1];
        self.get_regs(BME68xRegister::VariantId.into(), &mut data)?;
        self.variant_id = BME68xVariant::from(data[0]);
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
        let par_t1_f32 = f32::from(self.calib.par_t1);
        let par_t3_f32 = f32::from(self.calib.par_t3);
        let temp_f32 = cast_u2f32(temp_adc);

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
        let var2 = var1 * var1 * (f32::from(self.calib.par_p6) / 131072.0);
        let var2 = var2 + (var1 * (f32::from(self.calib.par_p5) * 2.0));
        let var2 = (var2 / 4.0) + ((f32::from(self.calib.par_p4)) * 65536.0);
        let var1 = (((f32::from(self.calib.par_p3) * var1 * var1) / 16384.0)
            + (f32::from(self.calib.par_p2) * var1))
            / 524288.0;
        let var1 = (1.0 + (var1 / 32768.0)) * (f32::from(self.calib.par_p1));
        // TODO: Might be worth looking to see if we can safe casting to float until later to reduce
        // operation of float
        let calc_pres = 1048576.0 - cast_u2f32(pres_adc);

        if var1 != 0.0 {
            let calc_pres = ((calc_pres - (var2 / 4096.0)) * 6250.0) / var1;
            let var1 = (f32::from(self.calib.par_p9) * calc_pres * calc_pres) / 2147483648.0;
            let var2 = calc_pres * (f32::from(self.calib.par_p8) / 32768.0);
            let var3 = (calc_pres / 256.0)
                * (calc_pres / 256.0)
                * (calc_pres / 256.0)
                * (f32::from(self.calib.par_p10) / 131072.0);
            calc_pres + (var1 + var2 + var3 + (f32::from(self.calib.par_p7) * 128.0)) / 16.0
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
        // TODO: Might be worth looking to see if we can safe casting to float until later to reduce
        // operation of float
        let var1 = cast_u2f32(hum_adc)
            - ((f32::from(self.calib.par_h1) * 16.0)
                + ((f32::from(self.calib.par_h3) / 2.0) * temp_comp));
        let var2 = var1
            * ((f32::from(self.calib.par_h2) / 262144.0)
                * (1.0
                    + ((f32::from(self.calib.par_h4) / 16384.0) * temp_comp)
                    + ((f32::from(self.calib.par_h5) / 1048576.0) * temp_comp * temp_comp)));
        let var3 = f32::from(self.calib.par_h6) / 16384.0;
        let var4 = f32::from(self.calib.par_h7) / 2097152.0;
        let calc_hum = var2 + ((var3 + (var4 * temp_comp)) * var2 * var2);

        if calc_hum > 100.0 {
            100.0
        } else if calc_hum < 0.0 {
            0.0
        } else {
            calc_hum
        }
    }

    /// Calculate gas resistance low value as a float
    ///
    /// # Arguments:
    /// * `gas_res_adc`: Raw ADC gas resistance value
    /// * `gas_range`: The gas range to use for the calculation
    fn calc_gas_resistance_low(&self, gas_res_adc: u16, gas_range: u8) -> f32 {
        let gas_res_f = f32::from(gas_res_adc);
        let gas_range_f = cast_i2f32(1 << gas_range);
        let lookup_k1_range = [
            0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -0.8, 0.0, 0.0, -0.2, -0.5, 0.0, -1.0, 0.0, 0.0,
        ];
        let lookup_k2_range = [
            0.0, 0.0, 0.0, 0.0, 0.1, 0.7, 0.0, -0.8, -0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];

        let var1 = 1340.0 + (5.0 * f32::from(self.calib.range_sw_err));
        let var2 = (var1) * (1.0 + lookup_k1_range[gas_range as usize] / 100.0);
        let var3 = 1.0 + (lookup_k2_range[gas_range as usize] / 100.0);

        1.0 / (var3 * (0.000000125) * gas_range_f * (((gas_res_f - 512.0) / var2) + 1.0))
    }

    /// Calculate the heater resistance using float
    ///
    /// # Arguments
    ///  * `temp`: The temperature
    fn calc_res_heat(&self, temp: u16) -> u8 {
        let temp = if temp > 400 { 400.0 } else { f32::from(temp) };

        let var1 = (f32::from(self.calib.par_gh1) / (16.0)) + 49.0;
        let var2 = ((f32::from(self.calib.par_gh2) / (32768.0)) * (0.0005)) + 0.00235;
        let var3 = f32::from(self.calib.par_gh3) / (1024.0);
        let var4 = var1 * (1.0 + (var2 * temp));
        let var5 = var4 + (var3 * f32::from(self.amb_temp));

        // Casting to u8 is deliberate here. The original library also does it.
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let result = (3.4
            * ((var5
                * (4.0 / (4.0 + f32::from(self.calib.res_heat_range)))
                * (1.0 / (1.0 + (f32::from(self.calib.res_heat_val) * 0.002))))
                - 25.0)) as u8;
        result
    }

    /// Read a single data from teh senssor
    fn read_field_data(&mut self, index: u8, data: &mut BME68xData) -> Result<(), BME68xError> {
        let mut tries = 5;
        while tries > 0 {
            let mut buff = [0; BME68X_LEN_FIELD];
            let reg_addr: u8 =
                (u8::from(BME68xRegister::Field0)) + (index * BME68X_LEN_FIELD_OFFSET);
            self.get_regs(reg_addr, &mut buff)?;

            data.status = buff[0] & BME68X_NEW_DATA_MSK;
            data.gas_index = buff[0] & BME68X_GAS_INDEX_MSK;
            data.meas_index = buff[1];

            let adc_pres =
                (u32::from(buff[2]) * 4096) | (u32::from(buff[3]) * 16) | (u32::from(buff[4]) / 16);
            let adc_temp =
                (u32::from(buff[5]) * 4096) | (u32::from(buff[6]) * 16) | (u32::from(buff[7]) / 16);
            let adc_hum = (u32::from(buff[8]) * 256) | u32::from(buff[9]);
            let adc_gas_res_low = (u32::from(buff[13]) * 4) | ((u32::from(buff[14])) / 64);
            let adc_gas_res_high = (u32::from(buff[15]) * 4) | ((u32::from(buff[16])) / 64);
            let gas_range_l = buff[14] & BME68X_GAS_RANGE_MSK;
            let gas_range_h = buff[16] & BME68X_GAS_RANGE_MSK;
            if matches!(self.variant_id, BME68xVariant::GasHigh) {
                data.status |= buff[16] & BME68X_GASM_VALID_MSK;
                data.status |= buff[16] & BME68X_HEAT_STAB_MSK;
            } else {
                data.status |= buff[14] & BME68X_GASM_VALID_MSK;
                data.status |= buff[14] & BME68X_HEAT_STAB_MSK;
            }

            if (data.status & BME68X_NEW_DATA_MSK) != 0 {
                let mut reg = [0; 1];
                self.get_regs(
                    (u8::from(BME68xRegister::ResHeat0)) + data.gas_index,
                    &mut reg,
                )?;
                data.res_heat = reg[0];

                self.get_regs(
                    (u8::from(BME68xRegister::IdacHeat0)) + data.gas_index,
                    &mut reg,
                )?;
                data.idac = reg[0];

                self.get_regs(
                    (u8::from(BME68xRegister::GasWait0)) + data.gas_index,
                    &mut reg,
                )?;
                data.gas_wait = reg[0];

                data.temperature = self.calc_temperature(adc_temp);
                data.pressure = self.calc_pressure(adc_pres);
                data.humidity = self.calc_humidity(adc_hum);
                if matches!(self.variant_id, BME68xVariant::GasHigh) {
                    data.gas_resistance = calc_gas_resistance_high(
                        // Checked mathmatically. Should never go out of bounds
                        u16::try_from(adc_gas_res_high)?,
                        gas_range_h,
                    );
                } else {
                    data.gas_resistance = self.calc_gas_resistance_low(
                        // Checked mathmatically. Should never go out of bounds
                        u16::try_from(adc_gas_res_low)?,
                        gas_range_l,
                    );
                }
                break;
            }
            (self.delay_us)(BME68X_PERIOD_POLL);
            tries -= 1;
        }
        Ok(())
    }

    /// Read all data fields of the sensor
    fn read_all_field_data(&mut self, data: &mut [BME68xData; 3]) -> Result<(), BME68xError> {
        let mut buff = [0; BME68X_LEN_FIELD * 3];
        let mut set_val = [0; 30];
        self.get_regs(BME68xRegister::Field0.into(), &mut buff)?;

        self.get_regs(BME68xRegister::IdacHeat0.into(), &mut set_val)?;

        for i in 0..3 {
            let off = i * BME68X_LEN_FIELD;
            data[i].status = buff[off] & BME68X_NEW_DATA_MSK;
            data[i].gas_index = buff[off] & BME68X_GAS_INDEX_MSK;
            data[i].meas_index = buff[off + 1];
            let adc_pres = (u32::from(buff[off + 2]) * 4096)
                | (u32::from(buff[off + 3]) * 16)
                | (u32::from(buff[off + 4]) / 16);
            let adc_temp = (u32::from(buff[off + 5]) * 4096)
                | (u32::from(buff[off + 6]) * 16)
                | (u32::from(buff[off + 7]) / 16);
            let adc_hum = (u32::from(buff[off + 8]) * 256) | u32::from(buff[off + 9]);
            let adc_gas_res_low =
                (u32::from(buff[off + 13]) * 4) | ((u32::from(buff[off + 14])) / 64);
            let adc_gas_res_high =
                (u32::from(buff[off + 15]) * 4) | ((u32::from(buff[off + 16])) / 64);
            let gas_range_l = buff[off + 14] & BME68X_GAS_RANGE_MSK;
            let gas_range_h = buff[off + 16] & BME68X_GAS_RANGE_MSK;
            if matches!(self.variant_id, BME68xVariant::GasHigh) {
                data[i].status |= buff[off + 16] & BME68X_GASM_VALID_MSK;
                data[i].status |= buff[off + 16] & BME68X_HEAT_STAB_MSK;
            } else {
                data[i].status |= buff[off + 14] & BME68X_GASM_VALID_MSK;
                data[i].status |= buff[off + 14] & BME68X_HEAT_STAB_MSK;
            }
            data[i].idac = set_val[usize::from(data[i].gas_index)];
            data[i].res_heat = set_val[usize::from(10 + data[i].gas_index)];
            data[i].gas_wait = set_val[usize::from(20 + data[i].gas_index)];
            data[i].temperature = self.calc_temperature(adc_temp);
            data[i].pressure = self.calc_pressure(adc_pres);
            data[i].humidity = self.calc_humidity(adc_hum);
            if matches!(self.variant_id, BME68xVariant::GasHigh) {
                data[i].gas_resistance = calc_gas_resistance_high(
                    // Checked mathmatically. Should never go out of bounds
                    u16::try_from(adc_gas_res_high)?,
                    gas_range_h,
                );
            } else {
                data[i].gas_resistance = self.calc_gas_resistance_low(
                    // Checked mathmatically. Should never go out of bounds
                    u16::try_from(adc_gas_res_low)?,
                    gas_range_l,
                );
            }
        }

        Ok(())
    }

    /// Switch between SPI memory pages
    fn set_mem_page(&mut self, reg_addr: u8) -> Result<(), BME68xError> {
        let mem_page = if reg_addr > 0x7f {
            BME68xMemPage::Page1
        } else {
            BME68xMemPage::Page0
        };

        // IDK why it is complaining here, I'm passing it in.
        #[allow(unused_variables)]
        let page_match = matches!(self.mem_page, mem_page);

        if !page_match {
            self.mem_page = mem_page;
            let write_buffer = [u8::from(BME68xRegister::MemPage) | BME68X_SPI_RD_MSK];
            let mut read_buffer = [0];
            let result = self
                .i2c
                .write_read(self.address.into(), &write_buffer, &mut read_buffer);
            if result.is_ok() {
                let reg = read_buffer[0] & (!BME68X_MEM_PAGE_MSK);
                let reg = reg | (u8::from(self.mem_page) & BME68X_MEM_PAGE_MSK);

                let write_buffer = [u8::from(BME68xRegister::MemPage) & BME68X_SPI_WR_MSK, reg];
                let result = self.i2c.write(self.address.into(), &write_buffer);
                if result.is_ok() {
                    self.intf_rslt = BME68xError::Ok;
                    Ok(())
                } else {
                    self.intf_rslt = BME68xError::ComFail;
                    Err(BME68xError::ComFail)
                }
            } else {
                self.intf_rslt = BME68xError::ComFail;
                Err(BME68xError::ComFail)
            }
        } else {
            Ok(())
        }
    }

    /// Get The current SPI memory page
    fn get_mem_page(&mut self) -> Result<(), BME68xError> {
        let mut read_buffer = [0];
        let result = self.i2c.write_read(
            self.address.into(),
            &[u8::from(BME68xRegister::MemPage) | BME68X_SPI_RD_MSK],
            &mut read_buffer,
        );
        if result.is_ok() {
            self.mem_page = BME68xMemPage::from(read_buffer[0] & BME68X_MEM_PAGE_MSK);
            Ok(())
        } else {
            Err(BME68xError::ComFail)
        }
    }

    /// Set heater configuration
    fn set_conf(
        &mut self,
        conf: &BME68xHeatrConf,
        op_mode: BME68xOpMode,
    ) -> Result<u8, BME68xError> {
        let mut nb_conv = 0;
        let mut write_len = 0;
        let mut rh_reg_addr = [0; 10];
        let mut rh_reg_data = [0; 10];
        let mut gw_reg_addr = [0; 10];
        let mut gw_reg_data = [0; 10];

        match op_mode {
            BME68xOpMode::ForcedMode => {
                rh_reg_addr[0] = BME68xRegister::ResHeat0.into();
                rh_reg_data[0] = self.calc_res_heat(conf.heatr_temp);
                gw_reg_addr[0] = BME68xRegister::GasWait0.into();
                gw_reg_data[0] = calc_gas_wait(conf.heatr_dur);
                nb_conv = 0;
                write_len = 1;
            }
            BME68xOpMode::SequentialMode => {
                for i in 0..conf.profile_len {
                    let index: usize = i.into();
                    rh_reg_addr[index] = u8::from(BME68xRegister::ResHeat0) + i;
                    rh_reg_data[index] = self.calc_res_heat(conf.heatr_temp_prof[index]);
                    gw_reg_addr[index] = u8::from(BME68xRegister::GasWait0) + i;
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
                    let index: usize = i.into();
                    rh_reg_addr[index] = u8::from(BME68xRegister::ResHeat0) + i;
                    rh_reg_data[index] = self.calc_res_heat(conf.heatr_temp_prof[index]);
                    gw_reg_addr[index] = u8::from(BME68xRegister::GasWait0) + i;
                    gw_reg_data[index] = calc_gas_wait(conf.heatr_dur_prof[index]);
                    nb_conv = conf.profile_len;
                    write_len = conf.profile_len;
                    let shared_dur = calc_heatr_dur_shared(conf.shared_heatr_dur);
                    self.set_regs(&[BME68xRegister::ShdHeatrDur.into()], &[shared_dur], 1)?;
                }
            }
            BME68xOpMode::SleepMode => return Err(BME68xError::DefineOpMode),
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
        // Will never go out of bounds, as `dur >= 0x783` is a check for a
        // smaller number than what could make this exceed u16
        dur = u16::try_from((u32::from(dur) * 1000) / 477).unwrap();
        while dur > 0x3F {
            dur >>= 2;
            factor += 1;
        }

        // Checked programatically. Does not look like this will ever go
        // out of bounds.
        heatdurval = u8::try_from(dur + (factor * 64)).unwrap();
    }

    heatdurval
}

// TODO: Document properly
/// Calcualte gas wait time
fn calc_gas_wait(mut dur: u16) -> u8 {
    let mut factor = 0;
    if dur >= 0xfc0 {
        0xff // Max Duration;
    } else {
        while dur > 0x3f {
            dur /= 4;
            factor += 1;
        }
        // Programatically checked. Should not overflow
        u8::try_from(dur + (factor * 64)).unwrap()
    }
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
            field.swap(low_index, high_index);
        }
    } else if (field[high_index].status & BME68X_NEW_DATA_MSK) != 0 {
        field.swap(low_index, high_index);
    }
}

/// Concatenate two u8 into a u16
///
/// # Arguments
/// * `msb`: The most significant bit
/// * `lsb`: The least significant bit
// FIXME: Convert this to a macro.
#[allow(clippy::similar_names)]
fn concat_bytes(msb: u8, lsb: u8) -> u16 {
    (u16::from(msb) << 8) | u16::from(lsb)
}

/// Set bits for a register
// FIXME: Convert this to a macro
fn set_bits(reg_data: u8, bitmask: u8, bitpos: u8, data: u8) -> u8 {
    (reg_data & !(bitmask)) | ((data << bitpos) & bitmask)
}

/// Set bits starting from position 0
fn set_bits_pos_0(reg_data: u8, bitmask: u8, data: u8) -> u8 {
    (reg_data & !(bitmask)) | (data & bitmask)
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
    let mut cent_res = 0.0;
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
    for entry in data {
        if (entry.status & BME68X_GASM_VALID_MSK) == 0 {
            return Err(BME68xError::SelfTest);
        }
    }

    if n_meas >= 6 {
        cent_res = (5.0 * (data[3].gas_resistance + data[5].gas_resistance))
            / (2.0 * data[4].gas_resistance);
    }
    if cent_res < 6.0 {
        return Err(BME68xError::SelfTest);
    }
    Ok(())
}

/// Calculate gas resistance high value as a float
///
/// # Arguments:
/// * `gas_res_adc`: Raw ADC gas resistance value
/// * `gas_range`: The gas range to use for the calculation
fn calc_gas_resistance_high(gas_res_adc: u16, gas_range: u8) -> f32 {
    let var1: u32 = 262144 >> gas_range;
    let var2: i32 = i32::from(gas_res_adc) - 512;
    let var2 = var2 * 3;
    let var2 = 4096 + var2;
    1000000.0 * cast_u2f32(var1) / cast_i2f32(var2)
}

/// Convert a u8 to an i8, allowing wrapping.
///
/// This exists to reduce the number of clippy errors in this file,
/// as this happens fairly frequently. If this function is used, it
/// indicates that the specific cast was checked and wrapping is intended.
///
/// # Arguments
/// * `value`: The value to convert
///
/// # Returns
/// The converted value
// TODO: Turn into a macro?
fn wrap_u2i8(value: u8) -> i8 {
    #[allow(clippy::cast_possible_wrap)]
    (value as i8)
}

/// Convert a u16 to an i16, allowing wrapping.
///
/// This exists to reduce the number of clippy errors in this file,
/// as this happens fairly frequently. If this function is used, it
/// indicates that the specific cast was checked and wrapping is intended.
///
/// # Arguments
/// * `value`: The value to convert
///
/// # Returns
/// The converted value
// TODO: Turn into a macro?
fn wrap_u2i16(value: u16) -> i16 {
    #[allow(clippy::cast_possible_wrap)]
    (value as i16)
}
/// Convert a u32 to an f32, allowing precision loss.
///
/// This exists to reduce the number of clippy errors in this file,
/// as this happens fairly frequently. If this function is used, it
/// indicates that the specific cast was checked and wrapping is intended.
///
/// # Arguments
/// * `value`: The value to convert
///
/// # Returns
/// The converted value
// TODO: Turn into a macro?
fn cast_u2f32(value: u32) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    (value as f32)
}
/// Convert a i32 to an f32, allowing precision loss.
///
/// This exists to reduce the number of clippy errors in this file,
/// as this happens fairly frequently. If this function is used, it
/// indicates that the specific cast was checked and wrapping is intended.
///
/// # Arguments
/// * `value`: The value to convert
///
/// # Returns
/// The converted value
// TODO: Turn into a macro?
fn cast_i2f32(value: i32) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    (value as f32)
}
