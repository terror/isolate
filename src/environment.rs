use super::*;

#[derive(Debug)]
pub struct Environment {
  /// First gid to use for sandboxes.
  ///
  /// The gids from `first_sandbox_gid` to `first_sandbox_gid + num_sandboxes` will be used for
  /// sandboxes.
  pub first_sandbox_gid: u32,

  /// First uid to use for sandboxes.
  ///
  /// The uids from `first_sandbox_uid` to `first_sandbox_uid + num_sandboxes` will be used for
  /// sandboxes.
  pub first_sandbox_uid: u32,

  /// Directory where lock files are created.
  ///
  /// This directory is created and verified upon `Sandbox` initialization.
  pub lock_root: PathBuf,

  /// Number of sandbox instances supported.
  pub num_sandboxes: u32,

  /// Only root can create new sandboxes (default: false, i.e., everybody can).
  pub restrict_initialization: bool,

  /// All sandboxes are created under this directory.
  ///
  /// To avoid symlink attacks, this directory and all its ancestors
  /// must be writeable only to root.
  ///
  /// This directory is created and verified upon `Sandbox` initialization.
  pub sandbox_root: PathBuf,
}

impl Default for Environment {
  fn default() -> Self {
    Self {
      first_sandbox_gid: 60000,
      first_sandbox_uid: 60000,
      lock_root: PathBuf::from("/run/isolate/locks"),
      num_sandboxes: 1000,
      restrict_initialization: false,
      sandbox_root: PathBuf::from("/var/local/lib/isolate"),
    }
  }
}
