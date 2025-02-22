use super::*;

#[derive(Debug)]
#[allow(unused)]
pub struct Credentials {
  /// The group id for the sandbox (`sandbox_first_gid` + `sandbox_id`).
  gid: Gid,
  /// The sandbox id (must be in the range [0, num_boxes - 1] inclusive).
  id: u32,
  /// Original group id that invoked the sandbox.
  original_gid: Gid,
  /// Original user id that invoked the sandbox.
  original_uid: Uid,
  /// The user id for the sandbox (`sandbox_first_uid` + `sandbox`).
  uid: Uid,
}

#[derive(Debug)]
#[allow(unused)]
pub struct Sandbox {
  /// The cgroup configuration for the sandbox.
  cgroup: Option<CgroupConfig>,

  /// Credentials for the sandbox.
  credentials: Credentials,

  /// The directory for the sandbox (`sandbox_root` / `sandbox_id`).
  directory: PathBuf,

  /// Whether the sandbox has been initialized.
  initialized: bool,

  /// Directory where lock files are created.
  ///
  /// This directory is created and verified upon `Sandbox` initialization.
  lock_root: PathBuf,

  /// All sandboxes are created under this directory.
  ///
  /// To avoid symlink attacks, this directory and all its ancestors
  /// must be writeable only to root.
  ///
  /// This directory is (optionally) created and verified upon `Sandbox` initialization.
  sandbox_root: PathBuf,

  /// Tell the sandbox manager to be verbose and report on what is going on.
  ///
  /// This is useful for debugging purposes.
  verbose: bool,

  /// Multiple instances of Isolate cannot manage the same sandbox
  /// simultaneously.
  ///
  /// If you attempt to do that, the new instance refuses to run.
  ///
  /// With this option, the new instance waits for the other instance to
  /// finish.
  wait: bool,
}

impl TryFrom<(Config, &Environment)> for Sandbox {
  type Error = Error;

  fn try_from(value: (Config, &Environment)) -> Result<Self> {
    let (config, environment) = value;

    Self::new(config, environment, &MaterialSystem)
  }
}

impl Sandbox {
  fn new(config: Config, environment: &Environment, system: &impl System) -> Result<Self> {
    ensure!(system.geteuid().is_root(), Error::NotRoot);

    if system.getegid().as_raw() != 0 {
      system
        .setegid(0)
        .map_err(|e| Error::Permission(format!("cannot switch to root group: {}", e)))?;
    }

    let id = config.sandbox_id.unwrap_or(0);

    ensure!(
      id < environment.num_sandboxes,
      Error::Config(format!(
        "sandbox id out of range (allowed: 0-{})",
        environment.num_sandboxes - 1
      ))
    );

    let original_uid = system.getuid();

    if environment.restrict_initialization {
      ensure!(
        original_uid.is_root(),
        Error::Permission("you must be root to initialize the sandbox".into())
      );
    }

    let (original_uid, original_gid) = config.credentials(system)?;

    // The umask is set to 0o022 to ensure that files created by the sandboxed process are
    // readable and writable by the user and group, but only readable by others.
    system.umask(Mode::from_bits_truncate(0o022));

    Ok(Self {
      cgroup: config.cgroup,
      credentials: Credentials {
        gid: (environment.first_sandbox_gid + id).into(),
        id,
        original_gid,
        original_uid,
        uid: (environment.first_sandbox_uid + id).into(),
      },
      directory: environment.sandbox_root.join(id.to_string()),
      initialized: true,
      lock_root: environment.lock_root.clone(),
      sandbox_root: environment.sandbox_root.clone(),
      verbose: config.verbose,
      wait: config.wait,
    })
  }

  /// Run a command in the sandbox.
  pub fn execute(&self, _ctx: ExecutionContext) -> Result<ExecutionResult> {
    ensure!(self.initialized, Error::NotInitialized);

    todo!("Run a specified command in the sandbox");
  }

  /// Clean up the sandbox.
  pub fn cleanup(&mut self) -> Result {
    ensure!(self.initialized, Error::NotInitialized);

    todo!("Clean up the sandbox");
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    assert_matches::assert_matches,
    config::Config,
    nix::{
      errno::Errno,
      sys::stat::Mode,
      unistd::{Gid, Uid},
    },
    std::cell::RefCell,
  };

  struct MockSystem {
    egid: Gid,
    euid: Uid,
    gid: Gid,
    setegid_errno: Option<Errno>,
    uid: Uid,
    umask: RefCell<Option<Mode>>,
  }

  impl System for MockSystem {
    fn getegid(&self) -> Gid {
      self.egid
    }

    fn geteuid(&self) -> Uid {
      self.euid
    }

    fn getgid(&self) -> Gid {
      self.gid
    }

    fn getuid(&self) -> Uid {
      self.uid
    }

    fn setegid(&self, _gid: u32) -> Result<(), nix::Error> {
      if let Some(errno) = self.setegid_errno {
        Err(errno)
      } else {
        Ok(())
      }
    }

    fn umask(&self, mask: Mode) -> Mode {
      *self.umask.borrow_mut() = Some(mask);
      Mode::from_bits_truncate(0)
    }
  }

  #[test]
  fn new_sandbox_without_root_euid() {
    let config = Config::default();

    let mock = MockSystem {
      egid: Gid::from_raw(0),
      euid: Uid::from_raw(1000), // The `euid` here is not root.
      gid: Gid::from_raw(0),
      setegid_errno: None,
      uid: Uid::from_raw(1000),
      umask: RefCell::new(None),
    };

    let result = Sandbox::new(config, &Environment::default(), &mock);

    assert!(matches!(result, Err(Error::NotRoot)));
  }

  #[test]
  fn new_sandbox_setegid_fails_with_eperm() {
    let config = Config::default();

    let mock = MockSystem {
      egid: Gid::from_raw(1000), // The `egid` here is not root.
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(1000),
      setegid_errno: Some(Errno::EPERM), // Used to simulate EPERM failure.
      uid: Uid::from_raw(0),
      umask: RefCell::new(None),
    };

    let result = Sandbox::new(config, &Environment::default(), &mock);

    assert_matches!(
      result,
      Err(Error::Permission(message)) if message.contains("cannot switch to root group")
    );
  }

  #[test]
  fn new_sandbox_setegid_fails_with_einval() {
    let config = Config::default();

    let mock = MockSystem {
      egid: Gid::from_raw(1000), // The `egid` here is not root.
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(1000),
      setegid_errno: Some(Errno::EINVAL), // Used to simulate EINVAL failure.
      uid: Uid::from_raw(0),
      umask: RefCell::new(None),
    };

    let result = Sandbox::new(config, &Environment::default(), &mock);

    assert_matches!(
      result,
      Err(Error::Permission(message)) if message.contains("cannot switch to root group")
    );
  }

  #[test]
  fn new_sandbox_as_uid_as_gid_non_root_original() {
    let config = Config {
      as_uid: Some(2000),
      as_gid: Some(2000),
      ..Default::default()
    };

    let mock = MockSystem {
      egid: Gid::from_raw(0),
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(1000),
      setegid_errno: None,
      uid: Uid::from_raw(1000), // The `uid` here is not root.
      umask: RefCell::new(None),
    };

    let result = Sandbox::new(config, &Environment::default(), &mock);

    assert_matches!(
      result,
      Err(Error::Permission(message)) if message.contains("you must be root to use `as_uid` or `as_gid`")
    );
  }

  #[test]
  fn new_sandbox_as_uid_without_as_gid() {
    let config = Config {
      as_uid: Some(2000),
      ..Default::default()
    };

    let mock = MockSystem {
      egid: Gid::from_raw(0),
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(0),
      setegid_errno: None,
      uid: Uid::from_raw(0),
      umask: RefCell::new(None),
    };

    let result = Sandbox::new(config, &Environment::default(), &mock);

    assert_matches!(
      result,
      Err(Error::Config(message)) if message.contains("`as_uid` and `as_gid` must be used either both or none")
    );
  }

  #[test]
  fn new_sandbox_valid_no_as() {
    let config = Config::default();

    let mock = MockSystem {
      egid: Gid::from_raw(0),
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(0),
      setegid_errno: None,
      uid: Uid::from_raw(0),
      umask: RefCell::new(None),
    };

    let sandbox = Sandbox::new(config, &Environment::default(), &mock)
      .expect("Sandbox creation should succeed");

    // With no as_uid/as_gid, the sandbox takes the original uid/gid from the system.
    assert_eq!(sandbox.credentials.original_gid, 0.into());
    assert_eq!(sandbox.credentials.original_uid, 0.into());

    assert_eq!(
      mock.umask.borrow().unwrap(),
      Mode::from_bits_truncate(0o022)
    );
  }

  #[test]
  fn new_sandbox_valid_with_as() {
    let config = Config {
      as_uid: Some(2000),
      as_gid: Some(2000),
      ..Default::default()
    };

    let mock = MockSystem {
      egid: Gid::from_raw(0),
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(0),
      setegid_errno: None,
      // In this scenario, the real uid/gid is root so using as_uid/as_gid is allowed.
      uid: Uid::from_raw(0),
      umask: RefCell::new(None),
    };

    let sandbox = Sandbox::new(config, &Environment::default(), &mock)
      .expect("Sandbox creation should succeed");

    // When as_uid/as_gid are provided and allowed, the sandbox's id are set to those values.
    assert_eq!(sandbox.credentials.original_uid, 2000.into());
    assert_eq!(sandbox.credentials.original_gid, 2000.into());
  }

  #[test]
  fn new_sandbox_box_dir_setup() {
    let environment = Environment {
      sandbox_root: PathBuf::from("/tmp/isolate_test"),
      first_sandbox_uid: 10000,
      first_sandbox_gid: 20000,
      num_sandboxes: 10,
      ..Default::default()
    };

    let config = Config {
      sandbox_id: Some(5),
      ..Default::default()
    };

    let mock = MockSystem {
      egid: Gid::from_raw(0),
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(0),
      setegid_errno: None,
      uid: Uid::from_raw(0),
      umask: RefCell::new(None),
    };

    let sandbox =
      Sandbox::new(config, &environment, &mock).expect("Sandbox creation should succeed");

    assert_eq!(
      sandbox.directory,
      PathBuf::from("/tmp/isolate_test").join("5")
    );

    assert_eq!(sandbox.credentials.gid, (20000 + 5).into());
    assert_eq!(sandbox.credentials.id, 5);
    assert_eq!(sandbox.credentials.uid, (10000 + 5).into());
  }

  #[test]
  fn new_sandbox_box_dir_out_of_range() {
    let environment = Environment {
      sandbox_root: PathBuf::from("/tmp/isolate_test"),
      first_sandbox_gid: 20000,
      first_sandbox_uid: 10000,
      num_sandboxes: 10, // Valid box id's are between 0 and 9 (inclusive).
      ..Default::default()
    };

    let config = Config {
      sandbox_id: Some(10),
      ..Default::default()
    };

    let mock = MockSystem {
      egid: Gid::from_raw(0),
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(0),
      setegid_errno: None,
      uid: Uid::from_raw(0),
      umask: RefCell::new(None),
    };

    let result = Sandbox::new(config, &environment, &mock);

    assert_matches!(
      result,
      Err(Error::Config(message)) if message.contains("sandbox id out of range")
    );
  }

  #[test]
  fn environment_restricts_sandbox_initialization() {
    let environment = Environment {
      num_sandboxes: 10,
      restrict_initialization: true,
      ..Default::default()
    };

    let config = Config {
      sandbox_id: Some(0),
      ..Default::default()
    };

    let mock = MockSystem {
      egid: Gid::from_raw(0),
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(0),
      setegid_errno: None,
      uid: Uid::from_raw(1000),
      umask: RefCell::new(None),
    };

    let result = Sandbox::new(config, &environment, &mock);

    assert_matches!(
      result,
      Err(Error::Permission(message)) if message.contains("you must be root to initialize the sandbox")
    );
  }
}
