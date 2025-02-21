use super::*;

#[derive(Debug)]
pub struct ExecutionContext<'a> {
  /// Arguments to pass to the program.
  pub arguments: Option<Vec<&'a str>>,

  /// Limit size of core files created when a process crashes to 'size'
  /// kilobytes.
  ///
  /// Defaults to zero, meaning that no core files are produced inside the
  /// sandbox.
  pub core_size_limit_kb: Option<u32>,

  /// When the `time` limit is exceeded, do not kill the program immediately,
  /// but wait until `extra_time` seconds elapse since the start of the
  /// program.
  ///
  /// This allows to report the real execution time, even if it exceeds the
  /// limit slightly.
  ///
  /// Fractional numbers are allowed.
  ///
  /// Defaults to 0.5 seconds.
  pub extra_time_ms: Option<f64>,

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
  ///
  /// Defaults to 8 MB.
  pub file_size_limit_kb: Option<u32>,

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
  pub memory_limit_kb: Option<u32>,

  /// Which directories to mount for this program.
  ///
  /// See `ExecutionContext::default_mounts` for the default set of mounts.
  mounts: Vec<Mount>,

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

  /// The program to run.
  ///
  /// This is the only required field.
  pub program: String,

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
  pub silent: bool,

  /// By default, Isolate removes all special files (other than regular files
  /// and directories) created inside the sandbox.
  ///
  /// If you need them, this option disables that behavior, but you need to
  /// carefully check what you open.
  pub special_files: bool,

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

  /// Limit process stack to 'size' kilobytes.
  ///
  /// By default, the whole address space is available for the stack, but it is
  /// subject to the `memory_limit` limit.
  ///
  /// If this limit is exceeded, the program receives the SIGSEGV signal.
  ///
  /// Defaults to 32 MB.
  pub stack_limit_kb: Option<u32>,

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

  /// Limit run time of the program to 'time' milliseconds.
  ///
  /// Fractional numbers are allowed.
  ///
  /// Time in which the OS assigns the processor to other tasks is not counted.
  ///
  /// If this limit is exceeded, the program is killed (after `extra_time`, if
  /// set).
  ///
  /// Defaults to 1 second.
  pub time_limit_ms: Option<f64>,

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

  /// Environment variables to pass to the program.
  ///
  /// If `inherit_env` is set to `true`, all environment variables from the parent are inherited,
  /// other variables specified by the user are added to the environment after.
  variables: Vec<Variable>,

  /// Limit wall-clock time to 'time' seconds.
  ///
  /// Fractional values are allowed.
  ///
  /// This clock measures the time from the start of the program to its exit,
  /// so it does not stop when the program has lost the CPU or when it is
  /// waiting for an external event.
  ///
  /// We recommend to use `time_limit` as the main limit, but set
  /// `wall_time_limit_ms` to a much higher value as a precaution against
  /// sleeping programs.
  ///
  /// If this limit is exceeded, the program is killed.
  ///
  /// Defaults to 5 seconds.
  pub wall_time_limit_ms: Option<f64>,

  /// Change directory to a specified path before executing the program.
  ///
  /// This path must be relative to the root of the sandbox.
  pub working_directory: Option<PathBuf>,
}

impl Default for ExecutionContext<'_> {
  fn default() -> Self {
    Self {
      arguments: None,
      core_size_limit_kb: Some(0),
      extra_time_ms: Some(0.5 * 1000.0),
      file_size_limit_kb: Some(8192),
      inherit_env: false,
      inherit_fds: false,
      memory_limit_kb: Some(256_000),
      mounts: Self::default_mounts().unwrap(),
      open_files_limit: Some(64),
      process_limit: Some(1),
      program: String::new(),
      share_net: false,
      silent: false,
      special_files: false,
      stack_limit_kb: Some(32_000),
      stderr: None,
      stderr_to_stdout: false,
      stdin: None,
      stdout: None,
      time_limit_ms: Some(1.0 * 1000.0),
      tty_hack: false,
      variables: Vec::new(),
      wall_time_limit_ms: Some(5.0 * 1000.0),
      working_directory: None,
    }
  }
}

impl<'a> ExecutionContext<'a> {
  pub fn new(program: String, arguments: Option<Vec<&'a str>>) -> Self {
    Self {
      program,
      arguments,
      ..Default::default()
    }
  }

  pub fn arguments(self, arguments: Option<Vec<&'a str>>) -> Self {
    Self { arguments, ..self }
  }

  pub fn core_size_limit_kb(self, core_size_limit_kb: u32) -> Self {
    Self {
      core_size_limit_kb: Some(core_size_limit_kb),
      ..self
    }
  }

  /// The sandboxed process gets its own filesystem namespace, which contains only paths
  /// specified by mount configurations.
  ///
  /// By default, all mounts are created read-only and restricted (no devices,
  /// no setuid binaries). This behavior can be modified using `MountOptions`.
  ///
  /// The default set of mounts includes:
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
  fn default_mounts() -> Result<Vec<Mount>> {
    Ok(vec![
      Mount::read_write("box", Some("./box"))?,
      Mount::read_only("bin", None::<&Path>)?,
      Mount::device("dev", None::<&Path>)?,
      Mount::read_only("lib", None::<&Path>)?,
      Mount::optional("lib64", None::<&Path>)?,
      Mount::filesystem("proc", "proc")?,
      Mount::temporary("tmp")?,
      Mount::read_only("usr", None::<&Path>)?,
    ])
  }

  pub fn extra_time_ms(self, extra_time_ms: f64) -> Self {
    Self {
      extra_time_ms: Some(extra_time_ms),
      ..self
    }
  }

  pub fn file_size_limit_kb(self, file_size_limit_kb: u32) -> Self {
    Self {
      file_size_limit_kb: Some(file_size_limit_kb),
      ..self
    }
  }

  pub fn inherit_env(self, inherit_env: bool) -> Self {
    Self {
      inherit_env,
      ..self
    }
  }

  pub fn inherit_fds(self, inherit_fds: bool) -> Self {
    Self {
      inherit_fds,
      ..self
    }
  }

  pub fn memory_limit_kb(self, memory_limit_kb: u32) -> Self {
    Self {
      memory_limit_kb: Some(memory_limit_kb),
      ..self
    }
  }

  /// Add a mount to the list of mounts.
  pub fn mount(self, mount: Mount) -> Self {
    Self {
      mounts: self.mounts.into_iter().chain(Some(mount)).collect(),
      ..self
    }
  }

  /// Replace the list of mounts with a new list.
  ///
  /// Care has to be taken to specify the correct set of
  /// mounts for the executed program to run correctly.
  ///
  /// In particular, +/box+ has to be bound.
  pub fn mounts(self, mounts: Vec<Mount>) -> Self {
    Self { mounts, ..self }
  }

  pub fn open_files_limit(self, open_files_limit: u32) -> Self {
    Self {
      open_files_limit: Some(open_files_limit),
      ..self
    }
  }

  pub fn process_limit(self, process_limit: u32) -> Self {
    Self {
      process_limit: Some(process_limit),
      ..self
    }
  }

  pub fn share_net(self, share_net: bool) -> Self {
    Self { share_net, ..self }
  }

  pub fn silent(self, silent: bool) -> Self {
    Self { silent, ..self }
  }

  pub fn special_files(self, special_files: bool) -> Self {
    Self {
      special_files,
      ..self
    }
  }

  pub fn stack_limit_kb(self, stack_limit_kb: u32) -> Self {
    Self {
      stack_limit_kb: Some(stack_limit_kb),
      ..self
    }
  }

  pub fn stderr(self, stderr: Option<PathBuf>) -> Self {
    Self { stderr, ..self }
  }

  pub fn stderr_to_stdout(self, stderr_to_stdout: bool) -> Self {
    Self {
      stderr_to_stdout,
      ..self
    }
  }

  pub fn stdin(self, stdin: Option<PathBuf>) -> Self {
    Self { stdin, ..self }
  }

  pub fn stdout(self, stdout: Option<PathBuf>) -> Self {
    Self { stdout, ..self }
  }

  pub fn time_limit_ms(self, time_limit_ms: f64) -> Self {
    Self {
      time_limit_ms: Some(time_limit_ms),
      ..self
    }
  }

  pub fn tty_hack(self, tty_hack: bool) -> Self {
    Self { tty_hack, ..self }
  }

  /// Add an environment variable to the list of environment variables.
  pub fn variable(self, variable: Variable) -> Self {
    Self {
      variables: self.variables.into_iter().chain(Some(variable)).collect(),
      ..self
    }
  }

  /// Replace the list of environment variables with a new list.
  pub fn variables(self, variables: Vec<Variable>) -> Self {
    Self { variables, ..self }
  }

  pub fn wall_time_limit_ms(self, wall_time_limit_ms: f64) -> Self {
    Self {
      wall_time_limit_ms: Some(wall_time_limit_ms),
      ..self
    }
  }

  pub fn working_directory(self, working_directory: Option<PathBuf>) -> Self {
    Self {
      working_directory,
      ..self
    }
  }
}
