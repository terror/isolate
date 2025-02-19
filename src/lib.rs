use {
  error::Error,
  nix::{
    sys::stat::{umask, Mode},
    unistd::{getegid, geteuid, getgid, getuid, setegid, Gid},
  },
  std::path::PathBuf,
  thiserror::Error,
};

mod error;

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct EnvironmentConfig {
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

impl Default for EnvironmentConfig {
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

#[derive(Debug)]
pub struct SandboxConfig {
  /// Sandbox behavior settings.
  pub behavior: BehaviorConfig,
  /// Control group configuration (optional).
  pub cgroup: Option<CgroupConfig>,
  /// Global environment configuration.
  pub environment: EnvironmentConfig,
  /// Filesystem and directory configuration.
  pub fs: FilesystemConfig,
  /// Input/Output configuration.
  pub io: IoConfig,
  /// Program execution limits and constraints.
  pub program: ProgramConfig,
  /// Core sandbox security and identity settings.
  pub security: SecurityConfig,
}

impl Default for SandboxConfig {
  fn default() -> Self {
    Self {
      behavior: BehaviorConfig::default(),
      cgroup: None,
      environment: EnvironmentConfig::default(),
      fs: FilesystemConfig::default(),
      io: IoConfig::default(),
      program: ProgramConfig::default(),
      security: SecurityConfig::default(),
    }
  }
}

#[derive(Debug)]
pub struct SecurityConfig {
  /// Act on behalf of the specified user ID (only if Isolate was invoked by
  /// root).
  ///
  /// This is used in scenarios where a root-controlled process
  /// manages creation of sandboxes for regular users, usually in conjunction
  /// with the `restricted_init` option in the configuration file.
  pub as_uid: Option<u32>,

  /// Act on behalf of the specified group ID (only if Isolate was invoked by
  /// root).
  ///
  /// This is used in scenarios where a root-controlled process
  /// manages creation of sandboxes for regular users, usually in conjunction
  /// with the `restricted_init` option in the configuration file.
  pub as_gid: Option<u32>,

  /// When you run multiple sandboxes in parallel,
  /// you have to assign unique IDs to them by this option.
  ///
  /// This defaults to 0.
  pub box_id: Option<u32>,

  /// By default, isolate closes all file descriptors passed from its parent
  /// except for descriptors 0, 1, and 2.
  ///
  /// This prevents unintentional descriptor leaks. In some cases, passing
  /// extra descriptors to the sandbox can be desirable, so you can use this
  /// switch to make them survive.
  pub inherit_fds: bool,

  /// By default, isolate creates a new network namespace for its child
  /// process.
  ///
  /// This namespace contains no network devices except for a
  /// per-namespace loopback.
  ///
  /// This prevents the program from communicating with the outside world.
  ///
  /// If you want to permit communication, you can use this switch to keep the
  /// child process in the parent's network namespace.
  pub share_net: bool,
}

impl Default for SecurityConfig {
  fn default() -> Self {
    Self {
      as_uid: None,
      as_gid: None,
      box_id: Some(0),
      inherit_fds: false,
      share_net: false,
    }
  }
}

#[derive(Debug)]
pub struct IoConfig {
  /// Redirect standard input from a file.
  ///
  /// The file has to be accessible inside the sandbox
  /// (which means that the sandboxed program can manipulate it arbitrarily).
  ///
  /// If not specified, standard input is inherited from the parent process.
  pub stdin: Option<PathBuf>,

  /// Redirect standard output to a file.
  ///
  /// The file has to be accessible inside the sandbox (which means that the
  /// sandboxed program can manipulate it arbitrarily).
  ///
  /// If not specified, standard output is inherited from the parent process
  /// and the sandbox manager does not write anything to it.
  pub stdout: Option<PathBuf>,

  /// Redirect standard error output to a file.
  ///
  /// The file has to be accessible inside the sandbox (which means that the
  /// sandboxed program can manipulate it arbitrarily).
  ///
  /// If not specified, standard error output is inherited from the parent
  /// process.
  ///
  /// See also `stderr-to-stdout`.
  pub stderr: Option<PathBuf>,

  /// Redirect standard error output to standard output.
  ///
  /// This is performed after the standard output is redirected by `stdout`.
  ///
  /// Mutually exclusive with `stderr`.
  pub stderr_to_stdout: bool,

  /// Try to handle interactive programs communicating over a tty.
  ///
  /// The sandboxed program will run in a separate process group, which will
  /// temporarily become the foreground process group of the terminal.
  ///
  /// When the program exits, the process group will be switched back to the
  /// caller.
  ///
  /// Please note that the program can do many nasty things including (but not
  /// limited to) changing terminal settings, changing the line discipline, and
  /// stuffing characters to the terminal's input queue using the TIOCSTI
  /// ioctl.
  ///
  /// Use with extreme caution.
  pub tty_hack: bool,
}

impl Default for IoConfig {
  fn default() -> Self {
    Self {
      stdin: None,
      stdout: None,
      stderr: None,
      stderr_to_stdout: false,
      tty_hack: false,
    }
  }
}

#[derive(Debug)]
pub struct FilesystemConfig {
  /// Change directory to a specified path before executing the program.
  ///
  /// This path must be relative to the root of the sandbox.
  pub cwd: Option<PathBuf>,

  /// Directory rules for mounting.
  pub dir_rules: Vec<DirRule>,

  /// Do not bind the default set of directories.
  ///
  /// Care has to be taken to specify the correct set of rules (using
  /// `dir_rules`) for the executed program to run correctly.
  ///
  /// In particular, +/box+ has to be bound.
  pub no_default_dirs: bool,

  /// By default, Isolate removes all special files (other than regular files
  /// and directories) created inside the sandbox.
  ///
  /// If you need them, this option disables that behavior, but you need to
  /// carefully check what you open.
  pub special_files: bool,
}

impl Default for FilesystemConfig {
  fn default() -> Self {
    Self {
      cwd: None,
      dir_rules: Vec::new(),
      no_default_dirs: false,
      special_files: false,
    }
  }
}

#[derive(Debug)]
pub struct BehaviorConfig {
  /// Inherit all variables from the parent.
  ///
  /// UNIX processes normally inherit all environment variables from their
  /// parent. The sandbox however passes only those variables which are
  /// explicitly requested by environment rules.
  pub full_env: bool,

  /// Tell the sandbox manager to keep silence.
  ///
  /// No status messages are printed to stderr except for fatal errors of the
  /// sandbox itself.
  ///
  /// The combination of `verbose` and `silent` has an undefined effect.
  pub silent: bool,

  /// Multiple instances of Isolate cannot manage the same sandbox
  /// simultaneously.
  ///
  /// If you attempt to do that, the new instance refuses to run.
  ///
  /// With this option, the new instance waits for the other instance to
  /// finish.
  pub wait: bool,

  /// Tell the sandbox manager to be verbose and report on what is going on.
  pub verbose: bool,
}

impl Default for BehaviorConfig {
  fn default() -> Self {
    Self {
      full_env: false,
      silent: false,
      verbose: false,
      wait: false,
    }
  }
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

/// Program execution configuration and resource limits
#[derive(Debug)]
pub struct ProgramConfig {
  /// Limit address space of the program to 'size' kilobytes.
  ///
  /// If more processes are allowed, this applies to each of them separately.
  ///
  /// If this limit is reached, further memory allocations fail (e.g., malloc
  /// returns NULL).
  pub memory_limit: Option<u32>,

  /// Limit run time of the program to 'time' seconds.
  ///
  /// Fractional numbers are allowed.
  ///
  /// Time in which the OS assigns the processor to other tasks is not counted.
  ///
  /// If this limit is exceeded, the program is killed (after `extra_time`, if
  /// set).
  pub time_limit: Option<f64>,

  /// Limit wall-clock time to 'time' seconds.
  ///
  /// Fractional values are allowed.
  ///
  /// This clock measures the time from the start of the program to its exit,
  /// so it does not stop when the program has lost the CPU or when it is
  /// waiting for an external event.
  ///
  /// We recommend to use `time_limit` as the main limit, but set
  /// `wall_time_limit` to a much higher value as a precaution against
  /// sleeping programs.
  ///
  /// If this limit is exceeded, the program is killed.
  pub wall_time_limit: Option<f64>,

  /// When the `time` limit is exceeded, do not kill the program immediately,
  /// but wait until `extra_time` seconds elapse since the start of the
  /// program.
  ///
  /// This allows to report the real execution time, even if it exceeds the
  /// limit slightly.
  ///
  /// Fractional numbers are allowed.
  pub extra_time: Option<f64>,

  /// Limit process stack to 'size' kilobytes.
  ///
  /// By default, the whole address space is available for the stack, but it is
  /// subject to the `memory_limit` limit.
  ///
  /// If this limit is exceeded, the program receives the SIGSEGV signal.
  pub stack_limit: Option<u32>,

  /// Limit number of open files to 'max'. The default value is 64. Setting
  /// this option to 0 will result in unlimited open files.
  ///
  /// If this limit is reached, system calls creating file descriptors fail
  /// with error EMFILE.
  pub open_files_limit: Option<u32>,

  /// Limit size of each file created (or modified) by the program to 'size'
  /// kilobytes.
  ///
  /// In most cases, it is better to restrict overall disk usage by a disk
  /// quota (see below).
  ///
  /// This option can help in cases when quotas are not enabled
  /// on the underlying filesystem.
  ///
  /// If this limit is reached, system calls expanding files fail with error
  /// EFBIG and the program receives the SIGXFSZ signal.
  pub file_size_limit: Option<u32>,

  /// Set disk quota to a given number of blocks. This requires the filesystem
  /// to be mounted with support for quotas.
  ///
  /// Unlike other options, this one must be given to *isolate --init*.
  ///
  /// Please note that this currently works only on the ext family of
  /// filesystems (other filesystems use other interfaces for setting
  /// quotas).
  ///
  /// If the quota is reached, system calls expanding files fail with error
  /// EDQUOT.
  pub block_quota: Option<u32>,

  /// Set disk quota to a given number of inodes.
  ///
  /// This requires the filesystem to be mounted with support for quotas.
  ///
  /// Unlike other options, this one must be given to *isolate --init*.
  ///
  /// Please note that this currently works only on the ext family of
  /// filesystems (other filesystems use other interfaces for setting
  /// quotas).
  ///
  /// If the quota is reached, system calls expanding files fail with error
  /// EDQUOT.
  pub inode_quota: Option<u32>,

  /// Limit size of core files created when a process crashes to 'size'
  /// kilobytes.
  ///
  /// Defaults to zero, meaning that no core files are produced inside the
  /// sandbox.
  pub core_size_limit: Option<u32>,

  /// Permit the program to create up to 'max' processes and/or threads.
  ///
  /// Please keep in mind that time and memory limit do not work with multiple
  /// processes unless you enable the control group mode.
  ///
  /// If 'max' is not given, an arbitrary number of processes can be run.
  ///
  /// By default, only one process is permitted.
  ///
  /// If this limit is exceeded, system calls creating processes fail with
  /// error EAGAIN.
  pub process_limit: Option<u32>,
}

impl Default for ProgramConfig {
  fn default() -> Self {
    Self {
      memory_limit: Some(256_000),
      time_limit: Some(1.0),
      wall_time_limit: Some(5.0),
      extra_time: Some(0.5),
      stack_limit: Some(32_000),
      open_files_limit: Some(64),
      file_size_limit: Some(8192),
      block_quota: None,
      inode_quota: None,
      core_size_limit: Some(0),
      process_limit: Some(1),
    }
  }
}

#[derive(Debug, Clone)]
pub struct CgroupConfig {
  /// CPU set configuration (e.g., "0-3,5,7")
  ///
  /// See linux/Documentation/cgroups/cpusets.txt for precise syntax.
  pub cpuset: Option<String>,

  /// Memory limit for the entire control group in kilobytes.
  pub memory_limit: Option<u32>,

  /// Memory nodes configuration (e.g., "0,1")
  ///
  /// See linux/Documentation/cgroups/cpusets.txt for precise syntax.
  pub memset: Option<String>,

  /// Control group under which we place our subgroups
  ///
  /// Either an explicit path to a subdirectory in cgroupfs, or "auto:file" to
  /// read the path from "file", where it is put by isolate-cg-helper.
  pub root: PathBuf,
}

impl Default for CgroupConfig {
  fn default() -> Self {
    Self {
      cpuset: None,
      memory_limit: None,
      memset: None,
      root: PathBuf::from("auto:/run/isolate/cgroup"),
    }
  }
}

#[derive(Debug, Default)]
pub struct Meta {
  /// When control groups are enabled, this is the total memory use
  /// by the whole control group (in kilobytes).
  ///
  /// If you use *isolate --run* multiple times in the same sandbox, the
  /// control group retains cached data from the previous runs, which also
  /// contributes to *cg-mem*.
  pub cg_mem: u32,

  /// Present when the program was killed by the out-of-memory killer
  /// (e.g., because it has exceeded the memory limit of its control group).
  ///
  /// This is reported only on Linux 4.13 and later.
  pub cg_oom_killed: bool,

  /// Number of context switches forced by the kernel.
  pub csw_forced: u32,

  /// Number of context switches caused by the process giving up the CPU
  pub csw_voluntary: u32,

  /// The program has exited normally with this exit code.
  pub exit_code: i32,

  /// The program has exited after receiving this fatal signal.
  pub exit_signal: i32,

  /// Present when the program was terminated by the sandbox
  /// (e.g., because it has exceeded the time limit).
  pub killed: bool,

  /// Maximum resident set size of the process (in kilobytes).
  pub max_rss: u32,

  /// Status message, not intended for machine processing.
  ///
  /// E.g., "Time limit exceeded."
  pub message: String,

  /// Two-letter status code:
  /// - *RE* -- run-time error, i.e., exited with a non-zero exit code
  /// - *SG* -- program died on a signal
  /// - *TO* -- timed out
  /// - *XX* -- internal error of the sandbox
  pub status: String,

  /// Run time of the program in fractional seconds.
  pub time: f64,

  /// Wall clock time of the program in fractional seconds.
  pub time_wall: f64,
}

#[derive(Debug)]
pub struct Sandbox {
  /// The current sandbox configuration.
  pub config: SandboxConfig,
  /// Whether the sandbox has been initialized.
  pub initialized: bool,
  /// Original group id that invoked the sandbox.
  pub original_gid: u32,
  /// Original user id that invoked the sandbox.
  pub original_uid: u32,
}

impl Sandbox {
  pub fn new(config: SandboxConfig) -> Result<Self> {
    if !geteuid().is_root() {
      return Err(Error::NotRoot);
    }

    if getegid().as_raw() != 0 {
      setegid(Gid::from_raw(0)).map_err(|e| {
        Error::PermissionError(format!("Cannot switch to root group: {}", e))
      })?;
    }

    let (original_uid, original_gid) = (getuid().as_raw(), getgid().as_raw());

    let (original_uid, original_gid) =
      match (config.security.as_uid, config.security.as_gid) {
        (Some(_), Some(_)) if !(original_uid == 0) => {
          return Err(Error::PermissionError(
            "you must be root to use `as_uid` or `as_gid`".into(),
          ));
        }
        (Some(uid), Some(gid)) => (uid, gid),
        (None, None) => (original_uid, original_gid),
        _ => {
          return Err(Error::ConfigError(
            "`as_uid` and `as_gid` must be used either both or none".into(),
          ))
        }
      };

    umask(Mode::from_bits_truncate(0o022));

    Ok(Self {
      config,
      initialized: false,
      original_gid,
      original_uid,
    })
  }

  pub fn add_dir_rule(&mut self, rule: DirRule) -> Result {
    if !self.initialized {
      return Err(Error::NotInitialized);
    }

    self.config.fs.dir_rules.push(rule);

    Ok(())
  }

  pub fn add_env_rule(&mut self, _var: &str, _value: Option<&str>) -> Result {
    if !self.initialized {
      return Err(Error::NotInitialized);
    }

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
  pub fn run(&self, _command: &str, _args: &[&str]) -> Result<Meta> {
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
  use super::*;

  #[test]
  fn sandbox_config_defaults() {
    let config = SandboxConfig::default();

    assert_eq!(config.security.box_id, Some(0));
    assert_eq!(config.program.open_files_limit, Some(64));
  }
}
