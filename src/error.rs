use super::*;

#[derive(Debug, Error)]
pub enum Error {
  #[error("sandbox has already been initialized")]
  AlreadyInitialized,
  #[error("bod id {0} out of range (allowed: 0-{1})")]
  BoxIdOutOfRange(u32, u32),
  #[error("configuration error: {0}")]
  Config(String),
  #[error("sandbox has not been initialized")]
  NotInitialized,
  #[error("operation requires root privileges")]
  NotRoot,
  #[error("permission error: {0}")]
  Permission(String),
}
