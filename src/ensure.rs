#[cfg(test)]
use super::*;

#[macro_export]
macro_rules! ensure {
  ($cond:expr, $err:expr) => {
    if !($cond) {
      return Err($err);
    }
  };
  ($cond:expr, $fmt:expr, $($arg:tt)*) => {
    if !($cond) {
      return Err($fmt.to_string(), $($arg)*);
    }
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ensures_properly() {
    fn validate_box_id(id: u32, max: u32) -> Result<()> {
      ensure!(id < max, Error::BoxIdOutOfRange(id, max - 1));

      Ok(())
    }

    fn validate_permission(is_root: bool) -> Result<()> {
      ensure!(
        is_root,
        Error::Permission("operation requires root privileges".into())
      );

      Ok(())
    }

    assert!(validate_box_id(5, 10).is_ok());

    assert!(matches!(
      validate_box_id(10, 10),
      Err(Error::BoxIdOutOfRange(10, 9))
    ));

    assert!(matches!(
      validate_permission(false),
      Err(Error::Permission(_))
    ));
  }
}
