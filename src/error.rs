use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
  #[error("sandbox has already been initialized")]
  AlreadyInitialized,
  #[error("box id {0} out of range (allowed: 0-{1})")]
  BoxIdOutOfRange(u32, u32),
  #[error("configuration error: {0}")]
  Config(String),
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
  #[error("invalid mount: {0}")]
  Mount(String),
  #[error("sandbox has not been initialized")]
  NotInitialized,
  #[error("operation requires root privileges")]
  NotRoot,
  #[error("permission error: {0}")]
  Permission(String),
}
