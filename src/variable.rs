use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
  Inherit,
  Clear,
  Set(String),
}

impl fmt::Display for Action {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Action::Inherit => write!(f, "inherit"),
      Action::Clear => write!(f, "clear"),
      Action::Set(value) => write!(f, "set({})", value),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
  pub key: String,
  pub action: Action,
}

impl Variable {
  pub fn new(key: impl Into<String>, action: Action) -> Self {
    Self {
      key: key.into(),
      action,
    }
  }

  pub fn get_value(&self) -> Option<&str> {
    match &self.action {
      Action::Set(value) => Some(value),
      _ => None,
    }
  }

  pub fn is_clear(&self) -> bool {
    matches!(self.action, Action::Clear)
  }

  pub fn is_inherit(&self) -> bool {
    matches!(self.action, Action::Inherit)
  }

  pub fn with_set_value(key: impl Into<String>, value: impl Into<String>) -> Self {
    Self {
      key: key.into(),
      action: Action::Set(value.into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn action_display() {
    assert_eq!(Action::Inherit.to_string(), "inherit");
    assert_eq!(Action::Clear.to_string(), "clear");
    assert_eq!(Action::Set("test".to_string()).to_string(), "set(test)");
  }

  #[test]
  fn variable_creation() {
    let var = Variable::new("KEY", Action::Inherit);
    assert_eq!(var.key, "KEY");
    assert_eq!(var.action, Action::Inherit);

    let var = Variable::with_set_value("KEY", "value");
    assert_eq!(var.key, "KEY");
    assert_eq!(var.action, Action::Set("value".to_string()));

    let string_key = String::from("KEY");
    let var = Variable::new(string_key, Action::Clear);
    assert_eq!(var.key, "KEY");
  }

  #[test]
  fn variable_state_checks() {
    let inherit_var = Variable::new("KEY", Action::Inherit);
    assert!(inherit_var.is_inherit());
    assert!(!inherit_var.is_clear());

    let clear_var = Variable::new("KEY", Action::Clear);
    assert!(clear_var.is_clear());
    assert!(!clear_var.is_inherit());

    let set_var = Variable::with_set_value("KEY", "value");
    assert!(!set_var.is_clear());
    assert!(!set_var.is_inherit());
  }

  #[test]
  fn get_value() {
    let inherit_var = Variable::new("KEY", Action::Inherit);
    assert_eq!(inherit_var.get_value(), None);

    let clear_var = Variable::new("KEY", Action::Clear);
    assert_eq!(clear_var.get_value(), None);

    let set_var = Variable::with_set_value("KEY", "value");
    assert_eq!(set_var.get_value(), Some("value"));
  }

  #[test]
  fn variable_equality() {
    let var1 = Variable::with_set_value("KEY", "value");
    let var2 = Variable::with_set_value("KEY", "value");
    let var3 = Variable::with_set_value("KEY", "different");

    assert_eq!(var1, var2);
    assert_ne!(var1, var3);
  }

  #[test]
  fn variable_cloning() {
    let original = Variable::with_set_value("KEY", "value");
    let cloned = original.clone();

    assert_eq!(original, cloned);

    // Ensure deep copy
    assert_eq!(original.get_value(), cloned.get_value());
  }
}
