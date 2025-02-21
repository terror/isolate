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
  pub file_size_limit_kb: Option<u32>,

  /// Limit address space of the program to 'size' kilobytes.
  ///
  /// If more processes are allowed, this applies to each of them separately.
  ///
  /// If this limit is reached, further memory allocations fail (e.g., malloc
  /// returns NULL).
  pub memory_limit_kb: Option<u32>,

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
  pub time_limit_ms: Option<f64>,

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
      memory_limit_kb: Some(256_000),
      open_files_limit: Some(64),
      process_limit: Some(1),
      program: String::new(),
      stack_limit_kb: Some(32_000),
      stderr: None,
      stderr_to_stdout: false,
      stdin: None,
      stdout: None,
      time_limit_ms: Some(1.0 * 1000.0),
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

  pub fn memory_limit_kb(self, memory_limit_kb: u32) -> Self {
    Self {
      memory_limit_kb: Some(memory_limit_kb),
      ..self
    }
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
