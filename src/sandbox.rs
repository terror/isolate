use super::*;

#[derive(Debug)]
#[allow(unused)]
pub struct Sandbox<'a> {
  /// The configuration for the sandbox.
  config: Config,
  /// The environment configuration.
  environment: &'a Environment,
  /// Whether the sandbox has been initialized.
  initialized: bool,
  /// Whether the sandbox was invoked by root.
  invoked_by_root: bool,
  /// Original group id that invoked the sandbox.
  original_gid: Gid,
  /// Original user id that invoked the sandbox.
  original_uid: Uid,
  /// The system to interact with.
  system: &'a dyn System,
}

impl<'a> TryFrom<(Config, &'a Environment)> for Sandbox<'a> {
  type Error = Error;

  fn try_from(value: (Config, &'a Environment)) -> Result<Self> {
    let (config, environment) = value;

    Self::new(config, environment, &MaterialSystem)
  }
}

impl<'a> Sandbox<'a> {
  fn new(config: Config, environment: &'a Environment, system: &'a dyn System) -> Result<Self> {
    ensure!(system.geteuid().is_root(), Error::NotRoot);

    if system.getegid().as_raw() != 0 {
      system
        .setegid(0)
        .map_err(|e| Error::Permission(format!("cannot switch to root group: {}", e)))?;
    }

    ensure!(
      config.sandbox_id.unwrap_or(0) < environment.num_sandboxes,
      Error::Config(format!(
        "sandbox id out of range (allowed: 0-{})",
        environment.num_sandboxes - 1
      ))
    );

    let (uid, gid) = (system.getuid(), system.getgid());

    let (original_uid, original_gid) = match (config.as_uid, config.as_gid) {
      (Some(_), Some(_)) if !uid.is_root() => Err(Error::Permission(
        "you must be root to use `as_uid` or `as_gid`".into(),
      )),
      (Some(as_uid), Some(as_gid)) => Ok((as_uid.into(), as_gid.into())),
      (None, None) => Ok((uid, gid)),
      _ => Err(Error::Config(
        "`as_uid` and `as_gid` must be used either both or none".into(),
      )),
    }?;

    system.umask(Mode::from_bits_truncate(0o022));

    Ok(Self {
      config,
      environment,
      initialized: false,
      invoked_by_root: uid.is_root(),
      original_gid,
      original_uid,
      system,
    })
  }

  /// Initialize the sandbox.
  ///
  /// This method should be called before executing any programs in the sandbox.
  pub fn initialize(&self) -> Result {
    if self.environment.restrict_initialization {
      ensure!(
        self.invoked_by_root,
        Error::Permission("you must be root to initialize the sandbox".into())
      );
    }

    if !self.environment.sandbox_root.exists() {
      self
        .system
        .create_directory(&self.environment.sandbox_root, 0o700)?;
    }

    for ancestor in self.environment.sandbox_root.ancestors() {
      let metadata = fs::metadata(ancestor)?;

      ensure!(
        metadata.permissions().mode() & 0o022 == 0,
        Error::Permission(format!(
          "directory {} must be writable only by root",
          ancestor.display()
        ))
      );

      ensure!(
        metadata.is_dir(),
        Error::Permission(format!("{} must be a directory", ancestor.display()))
      );
    }

    self.system.recreate_directory(&self.directory(), 0o700)?;

    let sandbox = self.directory().join("box");

    self.system.create_directory(&sandbox, 0o700)?;

    self
      .system
      .chown(&sandbox, Some(self.original_uid), Some(self.original_gid))
      .map_err(|error| Error::Permission(format!("cannot chown sandbox path: {error}")))?;

    Ok(())
  }

  /// Execute a program in the sandbox.
  pub fn execute(&self, _ctx: ExecutionContext) -> Result<ExecutionResult> {
    ensure!(self.initialized, Error::NotInitialized);

    todo!("Execute a specified program in the sandbox");
  }

  /// Clean up the sandbox.
  pub fn cleanup(&mut self) -> Result {
    ensure!(self.initialized, Error::NotInitialized);

    todo!("Clean up the sandbox");
  }

  /// Get the id of the sandbox.
  pub fn id(&self) -> u32 {
    self.config.sandbox_id.unwrap_or(0)
  }

  /// Get the group id of the sandbox.
  pub fn gid(&self) -> Gid {
    Gid::from_raw(self.environment.first_sandbox_gid + self.id())
  }

  /// Get the user id of the sandbox.
  pub fn uid(&self) -> Uid {
    Uid::from_raw(self.environment.first_sandbox_uid + self.id())
  }

  /// Get the directory of the sandbox.
  pub fn directory(&self) -> PathBuf {
    self.environment.sandbox_root.join(self.id().to_string())
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

  #[derive(Debug)]
  struct MockSystem {
    chown_errno: Option<Errno>,
    egid: Gid,
    euid: Uid,
    gid: Gid,
    setegid_errno: Option<Errno>,
    uid: Uid,
    umask: RefCell<Option<Mode>>,
  }

  impl Default for MockSystem {
    fn default() -> Self {
      Self {
        chown_errno: None,
        egid: Gid::from_raw(0),
        euid: Uid::from_raw(0),
        gid: Gid::from_raw(0),
        setegid_errno: None,
        uid: Uid::from_raw(0),
        umask: RefCell::new(None),
      }
    }
  }

  impl System for MockSystem {
    fn chown(&self, _path: &Path, _uid: Option<Uid>, _gid: Option<Gid>) -> Result<(), nix::Error> {
      if let Some(errno) = self.chown_errno {
        Err(errno)
      } else {
        Ok(())
      }
    }

    fn create_directory(&self, _path: &Path, _mode: u32) -> Result {
      Ok(())
    }

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

    fn recreate_directory(&self, _path: &Path, _mode: u32) -> Result {
      Ok(())
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
  fn sandbox_construction_without_root_euid() {
    let environment = Environment::default();

    let config = Config::default();

    let mock = MockSystem {
      euid: Uid::from_raw(1000),
      ..Default::default()
    };

    let result = Sandbox::new(config, &environment, &mock);

    assert!(matches!(result, Err(Error::NotRoot)));
  }

  #[test]
  fn sandbox_construction_setegid_fails_with_eperm() {
    let environment = Environment::default();

    let config = Config::default();

    let mock = MockSystem {
      egid: Gid::from_raw(1000),
      setegid_errno: Some(Errno::EPERM),
      ..Default::default()
    };

    let result = Sandbox::new(config, &environment, &mock);

    assert_matches!(
      result,
      Err(Error::Permission(message)) if message.contains("cannot switch to root group")
    );
  }

  #[test]
  fn sandbox_construction_setegid_fails_with_einval() {
    let environment = Environment::default();

    let config = Config::default();

    let mock = MockSystem {
      egid: Gid::from_raw(1000),
      setegid_errno: Some(Errno::EINVAL),
      ..Default::default()
    };

    let result = Sandbox::new(config, &environment, &mock);

    assert_matches!(
      result,
      Err(Error::Permission(message)) if message.contains("cannot switch to root group")
    );
  }

  #[test]
  fn sandbox_construction_as_uid_as_gid_non_root_original() {
    let environment = Environment::default();

    let config = Config {
      as_uid: Some(2000),
      as_gid: Some(2000),
      ..Default::default()
    };

    let mock = MockSystem {
      uid: Uid::from_raw(1000),
      ..Default::default()
    };

    let result = Sandbox::new(config, &environment, &mock);

    assert_matches!(
      result,
      Err(Error::Permission(message)) if message.contains("you must be root to use `as_uid` or `as_gid`")
    );
  }

  #[test]
  fn sandbox_construction_as_uid_without_as_gid() {
    let (mock, environment) = (MockSystem::default(), Environment::default());

    let config = Config {
      as_uid: Some(2000),
      ..Default::default()
    };

    let result = Sandbox::new(config, &environment, &mock);

    assert_matches!(
      result,
      Err(Error::Config(message)) if message.contains("`as_uid` and `as_gid` must be used either both or none")
    );
  }

  #[test]
  fn sandbox_construction_valid_no_as() {
    let (mock, environment) = (MockSystem::default(), Environment::default());

    let config = Config::default();

    let sandbox =
      Sandbox::new(config, &environment, &mock).expect("Sandbox creation should succeed");

    // With no as_uid/as_gid, the sandbox takes the original uid/gid from the system.
    assert_eq!(sandbox.original_gid, 0.into());
    assert_eq!(sandbox.original_uid, 0.into());

    assert_eq!(
      mock.umask.borrow().unwrap(),
      Mode::from_bits_truncate(0o022)
    );
  }

  #[test]
  fn sandbox_construction_valid_with_as() {
    let (mock, environment) = (MockSystem::default(), Environment::default());

    let config = Config {
      as_uid: Some(2000),
      as_gid: Some(2000),
      ..Default::default()
    };

    let sandbox =
      Sandbox::new(config, &environment, &mock).expect("Sandbox creation should succeed");

    // When as_uid/as_gid are provided and allowed, the sandbox's id are set to those values.
    assert_eq!(sandbox.original_uid, 2000.into());
    assert_eq!(sandbox.original_gid, 2000.into());
  }

  #[test]
  fn sandbox_construction_credentials_setup() {
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

    let mock = MockSystem::default();

    let sandbox =
      Sandbox::new(config, &environment, &mock).expect("Sandbox creation should succeed");

    assert_eq!(
      sandbox.directory(),
      PathBuf::from("/tmp/isolate_test").join("5")
    );

    assert_eq!(sandbox.gid(), (20000 + 5).into());
    assert_eq!(sandbox.id(), 5);
    assert_eq!(sandbox.uid(), (10000 + 5).into());
  }

  #[test]
  fn sandbox_construction_id_out_of_range() {
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

    let mock = MockSystem::default();

    let result = Sandbox::new(config, &environment, &mock);

    assert_matches!(
      result,
      Err(Error::Config(message)) if message.contains("sandbox id out of range")
    );
  }

  #[test]
  fn sandbox_initialization_restricted_by_environment() {
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
      uid: Uid::from_raw(1000),
      ..Default::default()
    };

    let sandbox = Sandbox::new(config, &environment, &mock).unwrap();

    assert_matches!(
      sandbox.initialize(),
      Err(Error::Permission(message)) if message.contains("you must be root to initialize the sandbox")
    );
  }
}
