use super::*;

#[derive(Debug, Default)]
pub struct DirOptions {
  /// Allow access to character and block devices.
  pub allow_devices: bool,

  /// Instead of binding a directory, mount a device-less filesystem called
  /// 'inside_path'.
  ///
  /// For example, this can be 'proc' or 'sysfs'.
  pub filesystem: Option<String>,

  /// Silently ignore the rule if the directory to be bound does not exist.
  pub maybe: bool,

  /// Disallow execution of binaries.
  pub no_exec: bool,

  /// Do not bind recursively.
  ///
  /// Without this option, mount points in the outside directory tree are
  /// automatically propagated to the sandbox.
  pub no_recursive: bool,

  /// Allow read-write access.
  pub read_write: bool,

  /// Bind a freshly created temporary directory writeable for the sandbox
  /// user.
  ///
  /// Accepts no 'outside_path', implies `rw`.
  pub temporary: bool,
}

#[derive(Debug)]
#[allow(unused)]
pub struct DirRule {
  /// Path inside the sandbox where the directory will be mounted.
  inside_path: PathBuf,
  /// Path outside the sandbox to be mounted.
  outside_path: Option<PathBuf>,
  /// Mount options for this directory.
  options: DirOptions,
}

impl DirRule {
  pub fn new(
    inside_path: impl AsRef<Path>,
    outside_path: Option<impl AsRef<Path>>,
    options: DirOptions,
  ) -> Result<Self> {
    let inside_path = inside_path.as_ref();

    if options.temporary && outside_path.is_some() {
      return Err(Error::DirRule(
        "temporary directory cannot have an outside path".to_string(),
      ));
    }

    let read_write = if options.temporary {
      true
    } else {
      options.read_write
    };

    Ok(Self {
      inside_path: inside_path.to_path_buf(),
      outside_path: outside_path.map(|p| p.as_ref().to_path_buf()),
      options: DirOptions {
        read_write,
        ..options
      },
    })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, assert_matches::assert_matches};

  #[test]
  fn valid_dir_rule() {
    let rule = DirRule::new(
      Path::new("/sandbox/path"),
      Some(Path::new("/host/path")),
      DirOptions::default(),
    );

    assert!(rule.is_ok());
  }

  #[test]
  fn temporary_with_outside_path() {
    let options = DirOptions {
      temporary: true,
      ..Default::default()
    };

    let rule = DirRule::new(
      Path::new("/sandbox/path"),
      Some(Path::new("/host/path")),
      options,
    );

    assert_matches!(
      rule,
      Err(Error::DirRule(message)) if message.contains("temporary directory cannot have an outside path")
    );
  }

  #[test]
  fn temporary_implies_read_write() {
    let options = DirOptions {
      temporary: true,
      read_write: false,
      ..Default::default()
    };

    let rule = DirRule::new(Path::new("/sandbox/path"), None::<&Path>, options);

    assert!(rule.is_ok());
    assert!(rule.unwrap().options.read_write);
  }
}
