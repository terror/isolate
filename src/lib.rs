use {
  config::Config,
  dir_rule::{DirOptions, DirRule},
  environment::Environment,
  error::Error,
  execution_result::ExecutionResult,
  nix::{
    sys::stat::{umask, Mode},
    unistd::{getegid, geteuid, getgid, getuid, setegid, Gid, Uid},
  },
  std::{
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
  },
  system::{MaterialSystem, System},
  thiserror::Error,
};

mod config;
mod dir_rule;
mod environment;
mod error;
mod execution_result;
mod system;

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
#[allow(unused)]
pub struct Sandbox {
  /// The directory for the sandbox (cf_box_root/<box_id>).
  box_dir: PathBuf,
  /// The group id for the sandbox (cf_first_gid + box_id).
  box_gid: u32,
  /// The sandbox ID (must be in the range 0..num_boxes).
  box_id: u32,
  /// The user id for the sandbox (cf_first_uid + box_id).
  box_uid: u32,
  /// Directory rules for mounting.
  dir_rules: Vec<DirRule>,
  /// Whether the sandbox has been initialized.
  initialized: bool,
  /// Original group id that invoked the sandbox.
  original_gid: u32,
  /// Original user id that invoked the sandbox.
  original_uid: u32,
}

impl Sandbox {
  pub fn new(config: Config, environment: Environment) -> Result<Self> {
    Self::with_system(config, environment, &MaterialSystem)
  }

  fn with_system(config: Config, environment: Environment, system: &impl System) -> Result<Self> {
    if !system.geteuid().is_root() {
      return Err(Error::NotRoot);
    }

    if system.getegid().as_raw() != 0 {
      system
        .setegid(0)
        .map_err(|e| Error::Permission(format!("cannot switch to root group: {}", e)))?;
    }

    let (original_uid, original_gid) = (system.getuid().as_raw(), system.getgid().as_raw());

    let (original_uid, original_gid) = match (config.as_uid, config.as_gid) {
      (Some(_), Some(_)) if original_uid != 0 => {
        return Err(Error::Permission(
          "you must be root to use `as_uid` or `as_gid`".into(),
        ));
      }
      (Some(uid), Some(gid)) => (uid, gid),
      (None, None) => (original_uid, original_gid),
      _ => {
        return Err(Error::Config(
          "`as_uid` and `as_gid` must be used either both or none".into(),
        ))
      }
    };

    system.umask(Mode::from_bits_truncate(0o022));

    let box_id = config.box_id.unwrap_or(0);

    if box_id >= environment.num_boxes {
      return Err(Error::Config(format!(
        "sandbox id out of range (allowed: 0-{})",
        environment.num_boxes - 1
      )));
    }

    // The sandboxed process gets its own filesystem namespace, which contains only subtrees
    // requested by directory rules.
    //
    // By default, all directories are bound read-only and restricted (no devices,
    // no setuid binaries). This behavior can be modified using the 'options':
    //
    // Unless `no_default_dirs` is specified, the default set of directory rules binds +/bin+,
    // +/dev+ (with devices allowed), +/lib+, +/lib64+ (if it exists), and +/usr+. It also binds
    // the working directory to +/box+ (read-write), mounts the proc filesystem at +/proc+, and
    // creates a temporary directory +/tmp+.
    //
    // The rules are executed in the order in which they are given. Default rules come before
    // all user rules. When a rule is replaced, it retains the original position
    // in the order. This matters when one rule's 'in' is a sub-directory of another
    // rule's 'in'. For example if you first bind to 'a' and then to 'a/b', it will work as
    // expected, but a sub-directory 'b' must have existed in the directory bound to 'a' (isolate
    // never creates subdirectories in bound directories for security reasons). If the
    // order is 'a/b' before 'a', then the directory bound to 'a/b' becomes invisible
    // by the later binding on 'a'.
    let default_dir_rules = vec![
      DirRule::new(
        "box",
        Some("./box"),
        DirOptions {
          read_write: true,
          ..Default::default()
        },
      )?,
      DirRule::new("bin", None::<&Path>, Default::default())?,
      DirRule::new(
        "dev",
        None::<&Path>,
        DirOptions {
          allow_devices: true,
          ..Default::default()
        },
      )?,
      DirRule::new("lib", None::<&Path>, Default::default())?,
      DirRule::new(
        "lib64",
        None::<&Path>,
        DirOptions {
          maybe: true,
          ..Default::default()
        },
      )?,
      DirRule::new(
        "proc",
        Some("proc"),
        DirOptions {
          filesystem: Some("proc".to_string()),
          ..Default::default()
        },
      )?,
      DirRule::new(
        "tmp",
        None::<&Path>,
        DirOptions {
          temporary: true,
          read_write: true,
          ..Default::default()
        },
      )?,
      DirRule::new("usr", None::<&Path>, Default::default())?,
    ];

    let dir_rules = config
      .no_default_dirs
      .then_some(default_dir_rules)
      .unwrap_or_default();

    Ok(Self {
      box_dir: environment.box_root.join(box_id.to_string()),
      box_gid: environment.first_gid + box_id,
      box_id,
      box_uid: environment.first_uid + box_id,
      dir_rules,
      initialized: false,
      original_gid,
      original_uid,
    })
  }

  pub fn add_dir_rule(&mut self, rule: DirRule) -> Result {
    self.dir_rules.push(rule);
    Ok(())
  }

  pub fn add_env_rule(&mut self, _var: &str, _value: Option<&str>) -> Result {
    todo!("Add environment rule to sandbox");
  }

  /// Initialize the sandbox with the current configuration.
  pub fn initialize(&mut self) -> Result<(), Error> {
    if self.initialized {
      return Err(Error::AlreadyInitialized);
    }

    todo!("Initialize the sandbox");
  }

  /// Run a command in the sandbox.
  pub fn run(&self, _command: &str, _args: &[&str]) -> Result<ExecutionResult> {
    if !self.initialized {
      return Err(Error::NotInitialized);
    }

    todo!("Run a specified command in the sandbox");
  }

  /// Clean up the sandbox.
  pub fn cleanup(&mut self) -> Result {
    if !self.initialized {
      return Err(Error::NotInitialized);
    }

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
    last_umask: RefCell<Option<Mode>>,
    setegid_errno: Option<Errno>,
    uid: Uid,
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
      *self.last_umask.borrow_mut() = Some(mask);
      Mode::from_bits_truncate(0)
    }
  }

  #[test]
  fn sandbox_config_defaults() {
    let config = Config::default();

    assert_eq!(config.box_id, Some(0));
    assert_eq!(config.open_files_limit, Some(64));
  }

  #[test]
  fn new_sandbox_without_root_euid() {
    let config = Config::default();

    let mock = MockSystem {
      euid: Uid::from_raw(1000), // The `euid` here is not root.
      egid: Gid::from_raw(0),
      uid: Uid::from_raw(1000),
      gid: Gid::from_raw(0),
      setegid_errno: None,
      last_umask: RefCell::new(None),
    };

    let result = Sandbox::with_system(config, Environment::default(), &mock);

    assert!(matches!(result, Err(Error::NotRoot)));
  }

  #[test]
  fn new_sandbox_setegid_fails_with_eperm() {
    let config = Config::default();

    let mock = MockSystem {
      euid: Uid::from_raw(0),
      egid: Gid::from_raw(1000), // The `egid` here is not root.
      uid: Uid::from_raw(0),
      gid: Gid::from_raw(1000),
      setegid_errno: Some(Errno::EPERM), // Used to simulate EPERM failure.
      last_umask: RefCell::new(None),
    };

    let result = Sandbox::with_system(config, Environment::default(), &mock);

    assert_matches!(
      result,
      Err(Error::Permission(message)) if message.contains("cannot switch to root group")
    );
  }

  #[test]
  fn new_sandbox_setegid_fails_with_einval() {
    let config = Config::default();

    let mock = MockSystem {
      euid: Uid::from_raw(0),
      egid: Gid::from_raw(1000), // The `egid` here is not root.
      uid: Uid::from_raw(0),
      gid: Gid::from_raw(1000),
      setegid_errno: Some(Errno::EINVAL), // Used to simulate EINVAL failure.
      last_umask: RefCell::new(None),
    };

    let result = Sandbox::with_system(config, Environment::default(), &mock);

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
      euid: Uid::from_raw(0),
      egid: Gid::from_raw(0),
      uid: Uid::from_raw(1000), // The `uid` here is not root.
      gid: Gid::from_raw(1000),
      setegid_errno: None,
      last_umask: RefCell::new(None),
    };

    let result = Sandbox::with_system(config, Environment::default(), &mock);

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
      last_umask: RefCell::new(None),
      setegid_errno: None,
      uid: Uid::from_raw(0),
    };

    let result = Sandbox::with_system(config, Environment::default(), &mock);

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
      last_umask: RefCell::new(None),
      setegid_errno: None,
      uid: Uid::from_raw(0),
    };

    let sandbox = Sandbox::with_system(config, Environment::default(), &mock)
      .expect("Sandbox creation should succeed");

    // With no as_uid/as_gid, the sandbox takes the original uid/gid from the system.
    assert_eq!(sandbox.original_gid, 0);
    assert_eq!(sandbox.original_uid, 0);

    assert_eq!(
      mock.last_umask.borrow().unwrap(),
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
      last_umask: RefCell::new(None),
      setegid_errno: None,
      // In this scenario, the real uid/gid is root so using as_uid/as_gid is allowed.
      uid: Uid::from_raw(0),
    };

    let sandbox = Sandbox::with_system(config, Environment::default(), &mock)
      .expect("Sandbox creation should succeed");

    // When as_uid/as_gid are provided and allowed, the sandbox's IDs are set to those values.
    assert_eq!(sandbox.original_uid, 2000);
    assert_eq!(sandbox.original_gid, 2000);
  }

  #[test]
  fn new_sandbox_box_dir_setup() {
    let environment = Environment {
      box_root: PathBuf::from("/tmp/isolate_test"),
      first_uid: 10000,
      first_gid: 20000,
      num_boxes: 10,
      ..Default::default()
    };

    let config = Config {
      box_id: Some(5),
      ..Default::default()
    };

    let mock = MockSystem {
      egid: Gid::from_raw(0),
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(0),
      last_umask: RefCell::new(None),
      setegid_errno: None,
      uid: Uid::from_raw(0),
    };

    let sandbox =
      Sandbox::with_system(config, environment, &mock).expect("Sandbox creation should succeed");

    assert_eq!(
      sandbox.box_dir,
      PathBuf::from("/tmp/isolate_test").join("5")
    );

    assert_eq!(sandbox.box_gid, 20000 + 5);
    assert_eq!(sandbox.box_id, 5);
    assert_eq!(sandbox.box_uid, 10000 + 5);
  }

  #[test]
  fn new_sandbox_box_dir_out_of_range() {
    let environment = Environment {
      box_root: PathBuf::from("/tmp/isolate_test"),
      first_gid: 20000,
      first_uid: 10000,
      num_boxes: 10, // Valid box ID's are between 0 and 9 (inclusive).
      ..Default::default()
    };

    let config = Config {
      box_id: Some(10),
      ..Default::default()
    };

    let mock = MockSystem {
      egid: Gid::from_raw(0),
      euid: Uid::from_raw(0),
      gid: Gid::from_raw(0),
      last_umask: RefCell::new(None),
      setegid_errno: None,
      uid: Uid::from_raw(0),
    };

    let result = Sandbox::with_system(config, environment, &mock);

    assert_matches!(
      result,
      Err(Error::Config(message)) if message.contains("sandbox id out of range")
    );
  }
}
