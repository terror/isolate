use std::path::PathBuf;

#[derive(Debug)]
pub struct SandboxConfig {
  /// Act on behalf of the specified user ID (only if Isolate was invoked by
  /// root). This is used in scenarios where a root-controlled process
  /// manages creation of sandboxes for regular users, usually in conjunction
  /// with the `restricted_init` option in the configuration file.
  pub as_uid: Option<u32>,

  /// Act on behalf of the specified group ID (only if Isolate was invoked by
  /// root). This is used in scenarios where a root-controlled process
  /// manages creation of sandboxes for regular users, usually in conjunction
  /// with the `restricted_init` option in the configuration file.
  pub as_gid: Option<u32>,

  /// When you run multiple sandboxes in parallel,
  /// you have to assign unique IDs to them by this option.
  ///
  /// This defaults to 0.
  pub box_id: Option<u32>,

  /// Change directory to 'dir' before executing the program.
  ///
  /// This path must be relative to the root of the sandbox.
  pub cwd: Option<PathBuf>,

  /// Inherit all variables from the parent.
  ///
  /// UNIX processes normally inherit all environment variables from their
  /// parent. The sandbox however passes only those variables which are
  /// explicitly requested by environment rules.
  pub full_env: bool,

  /// By default, isolate closes all file descriptors passed from its parent
  /// except for descriptors 0, 1, and 2.
  ///
  /// This prevents unintentional descriptor leaks. In some cases, passing
  /// extra descriptors to the sandbox can be desirable, so you can use this
  /// switch to make them survive.
  pub inherit_fds: bool,

  /// Do not bind the default set of directories.
  ///
  /// Care has to be taken to specify the correct set of rules (using *--dir*)
  /// for the executed program to run correctly.
  ///
  /// In particular, +/box+ has to be bound.
  pub no_default_dirs: bool,

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

  /// Redirect standard input from 'file'.
  ///
  /// The 'file' has to be accessible inside the sandbox
  /// (which means that the sandboxed program can manipulate it arbitrarily).
  ///
  /// If not specified, standard input is inherited from the parent process.
  pub stdin: Option<PathBuf>,

  // Redirect standard error output to 'file'.
  //
  // The 'file' has to be accessible inside the sandbox (which means that the
  // sandboxed program can manipulate it arbitrarily).
  //
  // If not specified, standard error output is inherited from the 	parent
  // process. See also *--stderr-to-stdout*.
  pub stdout: Option<PathBuf>,

  /// Tell the sandbox manager to keep silence.
  ///
  /// No status messages are printed to stderr except for fatal errors of the
  /// sandbox itself.
  ///
  /// The combination of `verbose` and `--silent` has an undefined effect.
  pub silent: bool,

  /// By default, Isolate removes all special files (other than regular files
  /// 	and directories) created inside the sandbox. If you need them, this
  /// option disables 	that behavior, but you need to carefully check what
  /// you open.
  pub special_files: bool,

  /// Redirect standard output to 'file'.
  ///
  /// The 'file' has to be accessible inside the sandbox (which means that the
  /// sandboxed program can manipulate 	it arbitrarily).
  ///
  /// If not specified, standard output is inherited from the parent process
  /// and the sandbox manager does not write anything to it.
  pub stderr: Option<PathBuf>,

  /// Redirect standard error output to standard output.
  ///
  /// This is performed after the standard output is redirected by `stdout`.
  ///
  ///	Mutually exclusive with `stderr`.
  pub stderr_to_stdout: bool,

  /// Try to handle interactive programs communicating over a tty.
  ///
  /// The sandboxed program will run in a separate process group, which will
  /// temporarily 	become the foreground process group of the terminal. When
  /// the program exits, the 	process group will be switched back to the
  /// caller. Please note that the program 	can do many nasty things
  /// including (but not limited to) changing terminal settings,
  /// changing the line discipline, and stuffing characters to the terminal's
  /// input queue using the TIOCSTI ioctl.
  ///
  /// Use with extreme caution.
  pub tty_hack: bool,

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

impl Default for SandboxConfig {
  fn default() -> Self {
    Self {
      as_uid: None,
      as_gid: None,
      box_id: Some(0),
      cwd: None,
      full_env: false,
      inherit_fds: false,
      no_default_dirs: false,
      share_net: false,
      stdin: None,
      stdout: None,
      silent: false,
      special_files: false,
      stderr: None,
      stderr_to_stdout: false,
      tty_hack: false,
      wait: false,
      verbose: false,
    }
  }
}

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
  /// If this limit is exceeded, the program is killed (after `extra-time`, if
  /// set).
  pub time_limit: Option<f64>,

  /// Limit wall-clock time to 'time' seconds.
  ///
  /// Fractional values are allowed.
  ///
  ///	This clock measures the time from the start of the program to its exit,
  ///	so it does not stop when the program has lost the CPU or when it is
  /// waiting 	for an external event.
  ///
  ///	We recommend to use `time_limit` as the main limit, but set
  /// `wall_time_limit` 	to a much higher value as a precaution against
  ///	sleeping programs.
  ///
  ///	If this limit is exceeded, the program is killed.
  pub wall_time_limit: Option<f64>,

  /// When the `time` limit is exceeded, do not kill the program immediately,
  /// but wait until `extra-time` seconds elapse since the start of the
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
  /// processes 	unless you enable the control group mode.
  ///
  ///	If 'max' is not given, an arbitrary number of processes can be run.
  ///
  ///	By default, only one process is permitted.
  ///
  ///	If this limit is exceeded, system calls creating processes fail with
  /// error EAGAIN.
  pub process_limit: Option<u32>,
}

#[derive(Debug, Default)]
pub struct Meta {
  /// When control groups are enabled, this is the total memory use
  ///	by the whole control group (in kilobytes).
  ///
  ///	If you use *isolate --run* multiple times in the same sandbox, the
  /// control group retains cached 	data from the previous runs, which also
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
  pub initialized: bool,
}

impl Sandbox {
  pub fn new(_config: SandboxConfig) -> Sandbox {
    Sandbox { initialized: false }
  }

  /// The sandboxed process gets its own filesystem namespace, which contains
  /// only subtrees requested by directory rules:
  ///
  /// *-d, --dir=*'in'*=*'out'[*:*'options']::
  /// 	Bind the directory 'out' as seen by the caller to the path 'in' inside
  /// the sandbox. If there already was a directory rule for 'in', it is
  /// replaced.
  ///
  /// *-d, --dir=*'dir'[*:*'options']::
  /// 	Bind the directory +/+'dir' to 'dir' inside the sandbox.
  /// 	If there already was a directory rule for 'in', it is replaced.
  ///
  /// *-d, --dir=*'in'*=*::
  /// 	Remove a directory rule for the path 'in' inside the sandbox.
  ///
  /// By default, all directories are bound read-only and restricted (no
  /// devices, no setuid binaries). This behavior can be modified using the
  /// 'options':
  ///
  /// *rw*::
  /// 	Allow read-write access.
  ///
  /// *dev*::
  /// 	Allow access to character and block devices.
  ///
  /// *noexec*::
  /// 	Disallow execution of binaries.
  ///
  /// *maybe*::
  /// 	Silently ignore the rule if the directory to be bound does not exist.
  ///
  /// *fs*::
  /// 	Instead of binding a directory, mount a device-less filesystem called
  /// 'in'. 	For example, this can be 'proc' or 'sysfs'.
  ///
  /// *tmp*::
  /// 	Bind a freshly created temporary directory writeable for the sandbox
  /// user. 	Accepts no 'out', implies *rw*.
  ///
  /// *norec*::
  /// 	Do not bind recursively. Without this option, mount points in the outside
  /// 	directory tree are automatically propagated to the sandbox.
  ///
  /// Unless `no_default_dirs` is specified, the default set of directory
  /// rules binds +/bin+, +/dev+ (with devices allowed), +/lib+, +/lib64+ (if
  /// it exists), and +/usr+. It also binds the working directory to +/box+
  /// (read-write), mounts the proc filesystem at +/proc+, and
  /// creates a temporary directory +/tmp+.
  ///
  /// The rules are executed in the order in which they are given. Default rules
  /// come before all user rules. When a rule is replaced, it retains the
  /// original position in the order. This matters when one rule's 'in' is a
  /// sub-directory of another rule's 'in'. For example if you first bind to
  /// 'a' and then to 'a/b', it will work as expected, but a sub-directory 'b'
  /// must have existed in the directory bound to 'a' (isolate never creates
  /// subdirectories in bound directories for security reasons). If the
  /// order is 'a/b' before 'a', then the directory bound to 'a/b' becomes
  /// invisible by the later binding on 'a'.
  pub fn add_dir_rule() {
    todo!()
  }

  /// UNIX processes normally inherit all environment variables from their
  /// parent. The sandbox however passes only those variables which are
  /// explicitly requested by environment rules:
  ///
  /// *-E, --env=*'var'::
  /// 	Inherit the variable 'var' from the parent.
  ///
  /// *-E, --env=*'var'*=*'value'::
  /// 	Set the variable 'var' to 'value'. When the 'value' is empty, the
  /// 	variable is removed from the environment.
  ///
  /// The rules are applied in the order in which they were given, except for
  /// `full-env`, which is applied first.
  ///
  /// The list of rules is automatically initialized with
  /// *-ELIBC_FATAL_STDERR_=1*.
  pub fn add_env_rule() {
    todo!()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn init() {
    let sandbox = Sandbox::new(SandboxConfig::default());
    assert_eq!(sandbox.initialized, false);
  }
}
