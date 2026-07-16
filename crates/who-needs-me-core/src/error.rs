use std::{fmt, io, path::PathBuf};

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug)]
pub enum CoreError {
    MissingHomeDirectory,
    MissingSessionDataRoot(String),
    MissingHookEventRoot(String),
    RootSetsDoNotMatch,
    InvalidSessionPath(PathBuf),
    DuplicateSession(String),
    ReadPath { path: PathBuf, source: io::Error },
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHomeDirectory => write!(formatter, "home directory is unavailable"),
            Self::MissingSessionDataRoot(provider) => {
                write!(formatter, "session data root is missing for {provider}")
            }
            Self::MissingHookEventRoot(provider) => {
                write!(formatter, "hook event root is missing for {provider}")
            }
            Self::RootSetsDoNotMatch => {
                write!(
                    formatter,
                    "session-data and hook-event providers must match"
                )
            }
            Self::InvalidSessionPath(path) => {
                write!(
                    formatter,
                    "session path has no usable file name: {}",
                    path.display()
                )
            }
            Self::DuplicateSession(key) => write!(formatter, "duplicate Session key: {key}"),
            Self::ReadPath { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for CoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadPath { source, .. } => Some(source),
            _ => None,
        }
    }
}
