//! Error structures for the application
use thiserror::Error;

/// Main error enumeration for the application
#[derive(Debug, Error)]
pub enum AppError {
    /// Could not convert from integer to enum
    #[error("Could not convert from enum to integer")]
    EnumConversionError,
}
