use super::*;

#[derive(Debug, Default)]
pub enum Status {
  /// Program exited with non-zero exit code.
  #[default]
  RuntimeError,
  /// Program terminated by signal.
  SignalError,
  /// Program exceeded time limit.
  Timeout,
  /// Internal sandbox error.
  InternalError,
}

impl Display for Status {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Status::RuntimeError => "RE",
        Status::SignalError => "SG",
        Status::Timeout => "TO",
        Status::InternalError => "XX",
      }
    )
  }
}

impl From<&str> for Status {
  fn from(s: &str) -> Self {
    match s {
      "RE" => Status::RuntimeError,
      "SG" => Status::SignalError,
      "TO" => Status::Timeout,
      "XX" => Status::InternalError,
      _ => Status::RuntimeError,
    }
  }
}

#[derive(Debug, Default)]
pub struct ExecutionResult {
  /// Total memory usage of the control group in kilobytes.
  ///
  /// Includes cached data from previous runs in the same sandbox.
  pub cgroup_memory_kb: u32,

  /// Number of involuntary context switches (forced by kernel).
  pub context_switches_forced: u32,

  /// Number of voluntary context switches (process yielded CPU).
  pub context_switches_voluntary: u32,

  /// CPU time used by the process in seconds.
  pub cpu_time_ms: f64,

  /// Process exit code (if terminated normally).
  pub exit_code: i32,

  /// Whether the program was terminated by the OOM killer.
  ///
  /// Only reported on Linux 4.13+.
  pub killed_by_oom: bool,

  /// Peak memory usage (resident set size) in kilobytes.
  pub peak_memory_kb: u32,

  /// Program's standard error.
  pub stderr: String,

  /// Program's standard output.
  pub stdout: String,

  /// Execution status code.
  pub status: Status,

  /// Human-readable status description (e.g., "Time limit exceeded").
  pub status_message: String,

  /// Whether the sandbox terminated the process (e.g., due to timeout).
  pub terminated_by_sandbox: bool,

  /// Signal that terminated the process (if killed by signal).
  pub termination_signal: i32,

  /// Total wall clock time in seconds.
  pub wall_time_ms: f64,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn status_display() {
    assert_eq!(Status::RuntimeError.to_string(), "RE");
    assert_eq!(Status::SignalError.to_string(), "SG");
    assert_eq!(Status::Timeout.to_string(), "TO");
    assert_eq!(Status::InternalError.to_string(), "XX");
  }

  #[test]
  fn status_from_str() {
    assert!(matches!(Status::from("RE"), Status::RuntimeError));
    assert!(matches!(Status::from("SG"), Status::SignalError));
    assert!(matches!(Status::from("TO"), Status::Timeout));
    assert!(matches!(Status::from("XX"), Status::InternalError));
    assert!(matches!(Status::from("invalid"), Status::RuntimeError));
  }
}
