use std::process::ExitCode as StdExitCode;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    InvalidArguments = 2,
    ConfigurationError = 3,
    DatabaseError = 4,
    ServiceManagerError = 5,
    PermissionDenied = 6,
    UnsupportedPlatform = 7,
}

impl From<ExitCode> for StdExitCode {
    fn from(code: ExitCode) -> Self {
        StdExitCode::from(code as u8)
    }
}

#[derive(Debug, Error)]
pub enum ByteLogError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Service manager error: {0}")]
    ServiceManager(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Unsupported platform or feature: {0}")]
    Unsupported(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unexpected error: {0}")]
    General(String),
}

pub type PcStatsError = ByteLogError;

impl ByteLogError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Database(_) => ExitCode::DatabaseError,
            Self::Config(_) => ExitCode::ConfigurationError,
            Self::Io(_) => ExitCode::GeneralError,
            Self::Serialization(_) => ExitCode::GeneralError,
            Self::Platform(_) => ExitCode::GeneralError,
            Self::ServiceManager(_) => ExitCode::ServiceManagerError,
            Self::PermissionDenied(_) => ExitCode::PermissionDenied,
            Self::Unsupported(_) => ExitCode::UnsupportedPlatform,
            Self::Validation(_) => ExitCode::InvalidArguments,
            Self::General(_) => ExitCode::GeneralError,
        }
    }
}

pub type Result<T> = std::result::Result<T, ByteLogError>;
