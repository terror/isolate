use super::*;

#[derive(Debug, Default, PartialEq)]
pub struct MountOptions {
  /// Allow access to character and block devices.
  pub allow_devices: bool,

  /// Instead of binding a directory, mount a device-less filesystem called
  /// `inside_path`.
  ///
  /// For example, this can be `proc` or `sysfs`.
  pub filesystem: Option<String>,

  /// Disallow execution of binaries.
  pub no_exec: bool,

  /// Do not bind recursively.
  ///
  /// Without this option, mount points in the outside directory tree are
  /// automatically propagated to the sandbox.
  pub no_recursive: bool,

  /// Silently ignore the mount if the directory to be bound does not exist.
  pub optional: bool,

  /// Allow read-write access.
  pub read_write: bool,

  /// Bind a freshly created temporary directory writeable for the sandbox
  /// user.
  ///
  /// Accepts no `outside_path`, implies `rw`.
  pub temporary: bool,
}

#[derive(Debug, Default, PartialEq)]
#[allow(unused)]
pub struct Mount {
  /// Path inside the sandbox where the directory will be mounted.
  inside_path: PathBuf,
  /// Path outside the sandbox to be mounted.
  outside_path: Option<PathBuf>,
  /// Mount options for this directory.
  options: MountOptions,
}

impl Mount {
  pub fn new(
    inside_path: impl AsRef<Path>,
    outside_path: Option<impl AsRef<Path>>,
    options: MountOptions,
  ) -> Result<Self> {
    let inside_path = inside_path.as_ref();

    if options.temporary && outside_path.is_some() {
      return Err(Error::Mount(
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
      options: MountOptions {
        read_write,
        ..options
      },
    })
  }

  pub fn read_only(inside: impl AsRef<Path>, outside: Option<impl AsRef<Path>>) -> Result<Self> {
    Self::new(inside, outside, MountOptions::default())
  }

  pub fn read_write(inside: impl AsRef<Path>, outside: Option<impl AsRef<Path>>) -> Result<Self> {
    Self::new(
      inside,
      outside,
      MountOptions {
        read_write: true,
        ..Default::default()
      },
    )
  }

  pub fn device(inside: impl AsRef<Path>, outside: Option<impl AsRef<Path>>) -> Result<Self> {
    Self::new(
      inside,
      outside,
      MountOptions {
        allow_devices: true,
        ..Default::default()
      },
    )
  }

  pub fn temporary(inside: impl AsRef<Path>) -> Result<Self> {
    Self::new(
      inside,
      None::<&Path>,
      MountOptions {
        temporary: true,
        read_write: true,
        ..Default::default()
      },
    )
  }

  pub fn filesystem(inside: impl AsRef<Path>, fs_type: impl Into<String>) -> Result<Self> {
    Self::new(
      inside,
      None::<&Path>,
      MountOptions {
        filesystem: Some(fs_type.into()),
        ..Default::default()
      },
    )
  }

  pub fn optional(inside: impl AsRef<Path>, outside: Option<impl AsRef<Path>>) -> Result<Self> {
    Self::new(
      inside,
      outside,
      MountOptions {
        optional: true,
        ..Default::default()
      },
    )
  }
}

#[cfg(test)]
mod tests {
  use {super::*, assert_matches::assert_matches};

  #[test]
  fn valid_mount() {
    let mount = Mount::new(
      Path::new("/sandbox/path"),
      Some(Path::new("/host/path")),
      MountOptions::default(),
    );

    assert!(mount.is_ok());
  }

  #[test]
  fn temporary_with_outside_path() {
    let options = MountOptions {
      temporary: true,
      ..Default::default()
    };

    let mount = Mount::new(
      Path::new("/sandbox/path"),
      Some(Path::new("/host/path")),
      options,
    );

    assert_matches!(
      mount,
      Err(Error::Mount(message)) if message.contains("temporary directory cannot have an outside path")
    );
  }

  #[test]
  fn temporary_implies_read_write() {
    let options = MountOptions {
      temporary: true,
      read_write: false,
      ..Default::default()
    };

    let mount = Mount::new(Path::new("/sandbox/path"), None::<&Path>, options);

    assert!(mount.is_ok());
    assert!(mount.unwrap().options.read_write);
  }

  #[test]
  fn mount_builders() {
    let rw = Mount::read_write("test", Some("/test")).unwrap();

    assert_eq!(
      rw.options,
      MountOptions {
        read_write: true,
        ..Default::default()
      }
    );

    let dev = Mount::device("dev", None::<&Path>).unwrap();

    assert_eq!(
      dev.options,
      MountOptions {
        allow_devices: true,
        ..Default::default()
      }
    );

    let tmp = Mount::temporary("tmp").unwrap();

    assert_eq!(
      tmp,
      Mount {
        inside_path: PathBuf::from("tmp"),
        outside_path: None,
        options: MountOptions {
          temporary: true,
          read_write: true,
          ..Default::default()
        }
      }
    );

    let fs = Mount::filesystem("proc", "proc").unwrap();

    assert_eq!(
      fs.options,
      MountOptions {
        filesystem: Some("proc".to_string()),
        ..Default::default()
      }
    );
  }
}
