use super::*;

#[derive(Debug, Error)]
pub enum Error {
  #[error("sandbox has not been initialized")]
  NotInitialized,
  #[error("sandbox has already been initialized")]
  AlreadyInitialized,
  #[error("invalid configuration: {0}")]
  InvalidConfig(String),
  #[error("invalid directory rule: {0}")]
  InvalidDirRule(String),
  #[error("invalid environment variable: {0}")]
  InvalidEnvVar(String),
  #[error("filesystem error: {0}")]
  FilesystemError(#[from] std::io::Error),
  #[error("mount error: {0}")]
  MountError(String),
  #[error("namespace error: {0}")]
  NamespaceError(String),
  #[error("permission denied: {0}")]
  PermissionDenied(String),
  #[error("cgroup error: {0}")]
  CgroupError(String),
  #[error("process error: {0}")]
  ProcessError(String),
  #[error("quota error: {0}")]
  QuotaError(String),
  #[error("resource limit error: {0}")]
  ResourceLimitError(String),
  #[error("system error: {0}")]
  SystemError(String),
}
