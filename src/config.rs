use super::*;

#[derive(Debug)]
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
  pub root: PathBuf,
}

impl Default for CgroupConfig {
  fn default() -> Self {
    Self {
      cpu_cores: None,
      memory_limit: None,
      memory_nodes: None,
      root: PathBuf::from("auto:/run/isolate/cgroup"),
    }
  }
}

#[derive(Debug)]
pub struct Config {
  /// Act on behalf of the specified group ID (only if Isolate was invoked by
  /// root).
  ///
  /// This is used in scenarios where a root-controlled process
  /// manages creation of sandboxes for regular users, usually in conjunction
  /// with the `restricted_init` option in the configuration file.
  pub as_gid: Option<u32>,

  /// Act on behalf of the specified user ID (only if Isolate was invoked by
  /// root).
  ///
  /// This is used in scenarios where a root-controlled process
  /// manages creation of sandboxes for regular users, usually in conjunction
  /// with the `restricted_init` option in the configuration file.
  pub as_uid: Option<u32>,

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

  /// When you run multiple sandboxes in parallel,
  /// you have to assign unique IDs to them by this option.
  ///
  /// This defaults to 0.
  pub box_id: Option<u32>,

  /// Control group configuration.
  pub cgroup: Option<CgroupConfig>,

  /// Limit size of core files created when a process crashes to 'size'
  /// kilobytes.
  ///
  /// Defaults to zero, meaning that no core files are produced inside the
  /// sandbox.
  pub core_size_limit: Option<u32>,

  /// When the `time` limit is exceeded, do not kill the program immediately,
  /// but wait until `extra_time` seconds elapse since the start of the
  /// program.
  ///
  /// This allows to report the real execution time, even if it exceeds the
  /// limit slightly.
  ///
  /// Fractional numbers are allowed.
  pub extra_time: Option<f64>,

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

  /// Limit address space of the program to 'size' kilobytes.
  ///
  /// If more processes are allowed, this applies to each of them separately.
  ///
  /// If this limit is reached, further memory allocations fail (e.g., malloc
  /// returns NULL).
  pub memory_limit: Option<u32>,

  /// Do not bind the default set of directories.
  ///
  /// Care has to be taken to specify the correct set of rules (using
  /// `dir_rules`) for the executed program to run correctly.
  ///
  /// In particular, +/box+ has to be bound.
  pub no_default_dirs: bool,

  /// Limit number of open files to 'max'. The default value is 64. Setting
  /// this option to 0 will result in unlimited open files.
  ///
  /// If this limit is reached, system calls creating file descriptors fail
  /// with error EMFILE.
  pub open_files_limit: Option<u32>,

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

  /// Limit process stack to 'size' kilobytes.
  ///
  /// By default, the whole address space is available for the stack, but it is
  /// subject to the `memory_limit` limit.
  ///
  /// If this limit is exceeded, the program receives the SIGSEGV signal.
  pub stack_limit: Option<u32>,

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

  /// Limit run time of the program to 'time' seconds.
  ///
  /// Fractional numbers are allowed.
  ///
  /// Time in which the OS assigns the processor to other tasks is not counted.
  ///
  /// If this limit is exceeded, the program is killed (after `extra_time`, if
  /// set).
  pub time_limit: Option<f64>,

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

  /// Change directory to a specified path before executing the program.
  ///
  /// This path must be relative to the root of the sandbox.
  pub working_directory: Option<PathBuf>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      as_gid: None,
      as_uid: None,
      block_quota: None,
      box_id: Some(0),
      cgroup: None,
      core_size_limit: Some(0),
      extra_time: Some(0.5),
      file_size_limit: Some(8192),
      inherit_env: false,
      inherit_fds: false,
      inode_quota: None,
      memory_limit: Some(256_000),
      no_default_dirs: false,
      open_files_limit: Some(64),
      process_limit: Some(1),
      share_net: false,
      silent: false,
      special_files: false,
      stack_limit: Some(32_000),
      stderr: None,
      stderr_to_stdout: false,
      stdin: None,
      stdout: None,
      time_limit: Some(1.0),
      tty_hack: false,
      verbose: false,
      wait: false,
      wall_time_limit: Some(5.0),
      working_directory: None,
    }
  }
}
