use super::*;

#[derive(Debug, PartialEq)]
pub enum CgroupRoot {
  Auto(PathBuf),
  Fixed(PathBuf),
}

impl Default for CgroupRoot {
  fn default() -> Self {
    Self::Auto(PathBuf::from("/run/isolate/cgroup"))
  }
}

impl From<PathBuf> for CgroupRoot {
  fn from(path: PathBuf) -> Self {
    if let Some(file_name) = path.to_str() {
      if let Some(stripped) = file_name.strip_prefix("auto:") {
        return Self::Auto(PathBuf::from(stripped));
      }
    }

    Self::Fixed(path)
  }
}

impl From<CgroupRoot> for PathBuf {
  fn from(root: CgroupRoot) -> Self {
    match root {
      CgroupRoot::Auto(path) => PathBuf::from(format!("auto:{}", path.display())),
      CgroupRoot::Fixed(path) => path,
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct CgroupConfig {
  /// Defines the CPU cores available for this control group using the cpuset format.
  ///
  /// For example, `"0-3,5,7"` restricts processes to cores 0, 1, 2, 3, 5, and 7.
  /// Refer to the [cpusets documentation](https://docs.kernel.org/admin-guide/cgroup-v1/cpusets.html) for valid syntax.
  pub cpu_cores: Option<String>,

  /// Specifies the maximum memory allocation for the control group, in kilobytes.
  ///
  /// This value limits the total memory usage of all tasks within the group.
  pub memory_limit: Option<u32>,

  /// Configures the memory nodes that this control group can access.
  ///
  /// For example, `"0,1"` allows usage of memory nodes 0 and 1.
  ///
  /// The format follows the same rules as for CPU sets; see the
  /// [cpusets documentation](https://docs.kernel.org/admin-guide/cgroup-v1/cpusets.html) for more details.
  pub memory_nodes: Option<String>,

  /// Specifies the root directory under which all subgroup control groups will be created.
  ///
  /// This can be either:
  /// - A fixed path in the cgroup filesystem, or
  /// - A dynamic path specified as `"auto:file"`, where the actual path is read from `file`
  pub root: CgroupRoot,
}

impl Default for CgroupConfig {
  fn default() -> Self {
    Self {
      cpu_cores: None,
      memory_limit: Some(1024 * 1024),
      memory_nodes: None,
      root: CgroupRoot::default(),
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct Config {
  /// Act on behalf of the specified group ID (only if Isolate was invoked by
  /// root).
  ///
  /// This is used in scenarios where a root-controlled process
  /// manages creation of sandboxes for regular users, usually in conjunction
  /// with the `restrict_initialization` option in the environment configuration.
  pub as_gid: Option<u32>,

  /// Act on behalf of the specified user ID (only if Isolate was invoked by
  /// root).
  ///
  /// This is used in scenarios where a root-controlled process
  /// manages creation of sandboxes for regular users, usually in conjunction
  /// with the `restrict_initialization` option in the environment configuration.
  pub as_uid: Option<u32>,

  /// Set disk quota to a given number of blocks. This requires the filesystem
  /// to be mounted with support for quotas.
  ///
  /// Please note that this currently works only on the ext family of
  /// filesystems (other filesystems use other interfaces for setting
  /// quotas).
  ///
  /// If the quota is reached, system calls expanding files fail with error
  /// EDQUOT.
  pub block_quota: Option<u32>,

  /// Control group configuration.
  pub cgroup: Option<CgroupConfig>,

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

  /// Inherit all variables from the parent.
  ///
  /// UNIX processes normally inherit all environment variables from their
  /// parent. The sandbox however passes only those variables which are
  /// explicitly requested by environment rules.
  pub inherit_env: bool,

  /// By default, isolate closes all file descriptors passed from its parent
  /// except for descriptors 0, 1, and 2.
  ///
  /// This prevents unintentional descriptor leaks. In some cases, passing
  /// extra descriptors to the sandbox can be desirable, so you can use this
  /// switch to make them survive.
  pub inherit_fds: bool,

  /// Do not mount the default set of directories.
  ///
  /// Care has to be taken to specify the correct set of
  /// mounts for the executed program to run correctly.
  ///
  /// In particular, +/box+ has to be bound.
  pub no_default_mounts: bool,

  /// When you run multiple sandboxes in parallel,
  /// you have to assign unique IDs to them by this option.
  ///
  /// This defaults to 0.
  pub sandbox_id: Option<u32>,

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

  /// Tell the sandbox manager to keep silence.
  ///
  /// No status messages are printed to stderr except for fatal errors of the
  /// sandbox itself.
  ///
  /// The combination of `verbose` and `silent` has an undefined effect.
  pub silent: bool,

  /// By default, Isolate removes all special files (other than regular files
  /// and directories) created inside the sandbox.
  ///
  /// If you need them, this option disables that behavior, but you need to
  /// carefully check what you open.
  pub special_files: bool,

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

  /// Tell the sandbox manager to be verbose and report on what is going on.
  pub verbose: bool,

  /// Multiple instances of Isolate cannot manage the same sandbox
  /// simultaneously.
  ///
  /// If you attempt to do that, the new instance refuses to run.
  ///
  /// With this option, the new instance waits for the other instance to
  /// finish.
  pub wait: bool,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      as_gid: None,
      as_uid: None,
      block_quota: None,
      cgroup: None,
      inherit_env: false,
      inherit_fds: false,
      inode_quota: None,
      no_default_mounts: false,
      sandbox_id: Some(0),
      share_net: false,
      silent: false,
      special_files: false,
      tty_hack: false,
      verbose: false,
      wait: false,
    }
  }
}

impl Config {
  /// Resolves effective user and group IDs for the sandbox.
  ///
  /// Returns either the current process's user/group IDs if as_uid/as_gid are `None`,
  /// or the specified as_uid/as_gid if the process has root privileges.
  pub fn credentials(&self, system: &impl System) -> Result<(u32, u32)> {
    let (uid, gid) = (system.getuid().as_raw(), system.getgid().as_raw());

    match (self.as_uid, self.as_gid) {
      (Some(_), Some(_)) if uid != 0 => Err(Error::Permission(
        "you must be root to use `as_uid` or `as_gid`".into(),
      )),
      (Some(as_uid), Some(as_gid)) => Ok((as_uid, as_gid)),
      (None, None) => Ok((uid, gid)),
      _ => Err(Error::Config(
        "`as_uid` and `as_gid` must be used either both or none".into(),
      )),
    }
  }

  /// The sandboxed process gets its own filesystem namespace, which contains only paths
  /// specified by mount configurations.
  ///
  /// By default, all mounts are created read-only and restricted (no devices,
  /// no setuid binaries). This behavior can be modified using `MountOptions`.
  ///
  /// Unless `no_default_dirs` is specified, the default set of mounts includes:
  /// - `/bin` (read-only)
  /// - `/dev` (with devices allowed)
  /// - `/lib` (read-only)
  /// - `/lib64` (read-only, optional)
  /// - `/usr` (read-only)
  /// - `/box` (read-write, bound to working directory)
  /// - `/proc` (proc filesystem)
  /// - `/tmp` (temporary directory, read-write)
  ///
  /// Mounts are processed in the order they are specified, with default mounts preceding
  /// user-defined ones. When a mount is replaced, it maintains its original position
  /// in the sequence.
  ///
  /// This ordering is significant when one mount's `inside_path` is a subdirectory of another
  /// mount's `inside_path`.
  ///
  /// For example, mounting "a" followed by "a/b" works as expected, but subdirectory "b" must exist
  /// in the directory mounted at "a" (the sandbox never creates subdirectories in mounted
  /// directories for security).
  ///
  /// If "a/b" is mounted before "a", the mount at "a/b" becomes inaccessible due to
  /// being overshadowed by the mount at "a".
  pub fn default_mounts(&self) -> Result<Vec<Mount>> {
    Ok(
      self
        .no_default_mounts
        .then_some(vec![
          Mount::read_write("box", Some("./box"))?,
          Mount::read_only("bin", None::<&Path>)?,
          Mount::device("dev", None::<&Path>)?,
          Mount::read_only("lib", None::<&Path>)?,
          Mount::optional("lib64", None::<&Path>)?,
          Mount::filesystem("proc", "proc")?,
          Mount::temporary("tmp")?,
          Mount::read_only("usr", None::<&Path>)?,
        ])
        .unwrap_or_default(),
    )
  }
}

#[cfg(test)]
mod tests {
  use {super::*, assert_matches::assert_matches};

  #[test]
  fn cgroup_root_from_pathbuf() {
    let (auto_path, fixed_path) = (
      PathBuf::from("auto:/some/path"),
      PathBuf::from("/some/fixed/path"),
    );

    assert_matches!(CgroupRoot::from(auto_path),
      CgroupRoot::Auto(path) if path == PathBuf::from("/some/path")
    );

    assert_matches!(CgroupRoot::from(fixed_path),
      CgroupRoot::Fixed(path) if path == PathBuf::from("/some/fixed/path")
    );
  }

  #[test]
  fn pathbuf_from_cgroup_root() {
    let (auto_root, fixed_root) = (
      CgroupRoot::Auto(PathBuf::from("/some/path")),
      CgroupRoot::Fixed(PathBuf::from("/some/fixed/path")),
    );

    let auto_path: PathBuf = auto_root.into();
    let fixed_path: PathBuf = fixed_root.into();

    assert_eq!(auto_path, PathBuf::from("auto:/some/path"));
    assert_eq!(fixed_path, PathBuf::from("/some/fixed/path"));
  }

  #[test]
  fn default_cgroup_config() {
    let config = CgroupConfig::default();

    assert_matches!(config.root,
      CgroupRoot::Auto(path) if path == PathBuf::from("/run/isolate/cgroup")
    );
  }
}
