use super::*;

#[derive(Debug, Default)]
pub struct DirOptions {
  /// Allow read-write access.
  pub read_write: bool,

  /// Allow access to character and block devices.
  pub allow_devices: bool,

  /// Disallow execution of binaries.
  pub no_exec: bool,

  /// Silently ignore the rule if the directory to be bound does not exist.
  pub maybe: bool,

  /// Instead of binding a directory, mount a device-less filesystem called
  /// 'inside_path'.
  ///
  /// For example, this can be 'proc' or 'sysfs'.
  pub filesystem: Option<String>,

  /// Bind a freshly created temporary directory writeable for the sandbox
  /// user.
  ///
  /// Accepts no 'outside_path', implies `rw`.
  pub temporary: bool,

  /// Do not bind recursively.
  ///
  /// Without this option, mount points in the outside directory tree are
  /// automatically propagated to the sandbox.
  pub no_recursive: bool,
}

#[derive(Debug)]
pub struct DirRule {
  /// Path inside the sandbox where the directory will be mounted.
  pub inside_path: PathBuf,
  /// Path outside the sandbox to be mounted.
  pub outside_path: Option<PathBuf>,
  /// Mount options for this directory.
  pub options: DirOptions,
}
