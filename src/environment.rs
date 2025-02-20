use super::*;

#[derive(Debug)]
pub struct Environment {
  /// All sandboxes are created under this directory.
  ///
  /// To avoid symlink attacks, this directory and all its ancestors
  /// must be writeable only to root.
  pub box_root: PathBuf,

  /// First GID to use for sandboxes.
  pub first_gid: u32,

  /// First UID to use for sandboxes.
  pub first_uid: u32,

  /// Directory where lock files are created.
  pub lock_root: PathBuf,

  /// Number of sandbox instances supported.
  pub num_boxes: u32,

  /// Only root can create new sandboxes (default: false, i.e., everybody can).
  pub restricted_init: bool,
}

impl Default for Environment {
  fn default() -> Self {
    Self {
      box_root: PathBuf::from("/var/local/lib/isolate"),
      first_gid: 60000,
      first_uid: 60000,
      lock_root: PathBuf::from("/run/isolate/locks"),
      num_boxes: 1000,
      restricted_init: false,
    }
  }
}
