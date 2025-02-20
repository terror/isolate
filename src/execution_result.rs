#[derive(Debug, Default)]
pub struct ExecutionResult {
  /// When control groups are enabled, this is the total memory use
  /// by the whole control group (in kilobytes).
  ///
  /// If you use `run` multiple times in the same sandbox, the control group
  /// retains cached data from the previous runs, which also contributes to `cg-mem`.
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
  ///
  /// - `RE` = run-time error, i.e., exited with a non-zero exit code.
  /// - `SG` = program died on a signal.
  /// - `TO` = timed out.
  /// - `XX` = internal error of the sandbox.
  pub status: String,

  /// Run time of the program in fractional seconds.
  pub time: f64,

  /// Wall clock time of the program in fractional seconds.
  pub time_wall: f64,
}
