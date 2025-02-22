use super::*;

#[derive(Debug, PartialEq)]
pub enum CgroupRoot {
  Automatic(PathBuf),
  Manual(PathBuf),
}

impl Default for CgroupRoot {
  fn default() -> Self {
    Self::Automatic(PathBuf::from("/run/isolate/cgroup"))
  }
}

impl From<PathBuf> for CgroupRoot {
  fn from(path: PathBuf) -> Self {
    if let Some(file_name) = path.to_str() {
      if let Some(stripped) = file_name.strip_prefix("auto:") {
        return Self::Automatic(PathBuf::from(stripped));
      }
    }

    Self::Manual(path)
  }
}

impl From<CgroupRoot> for PathBuf {
  fn from(root: CgroupRoot) -> Self {
    match root {
      CgroupRoot::Automatic(path) => PathBuf::from(format!("auto:{}", path.display())),
      CgroupRoot::Manual(path) => path,
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
  ///
  /// The default value is `"auto:/run/isolate/cgroup"`.
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
  /// Act on behalf of the specified group id (only if Isolate was invoked by
  /// root).
  ///
  /// This is used in scenarios where a root-controlled process
  /// manages creation of sandboxes for regular users, usually in conjunction
  /// with the `restrict_initialization` option in the environment configuration.
  pub as_gid: Option<u32>,

  /// Act on behalf of the specified user id (only if Isolate was invoked by
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

  /// When you run multiple sandboxes in parallel,
  /// you have to assign unique id's to them by this option.
  ///
  /// This defaults to 0.
  pub sandbox_id: Option<u32>,

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
      inode_quota: None,
      sandbox_id: Some(0),
      verbose: false,
      wait: false,
    }
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
      CgroupRoot::Automatic(path) if path == PathBuf::from("/some/path")
    );

    assert_matches!(CgroupRoot::from(fixed_path),
      CgroupRoot::Manual(path) if path == PathBuf::from("/some/fixed/path")
    );
  }

  #[test]
  fn pathbuf_from_cgroup_root() {
    let (auto_root, fixed_root) = (
      CgroupRoot::Automatic(PathBuf::from("/some/path")),
      CgroupRoot::Manual(PathBuf::from("/some/fixed/path")),
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
      CgroupRoot::Automatic(path) if path == PathBuf::from("/run/isolate/cgroup")
    );
  }
}
