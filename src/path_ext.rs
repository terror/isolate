use super::*;

pub trait PathExt {
  fn create(&self, mode: u32) -> Result;
  fn recreate(&self, mode: u32) -> Result;
}

impl PathExt for PathBuf {
  fn create(&self, mode: u32) -> Result {
    fs::create_dir_all(self)?;
    fs::set_permissions(self, fs::Permissions::from_mode(mode))?;
    Ok(())
  }

  fn recreate(&self, mode: u32) -> Result {
    if self.exists() {
      fs::remove_dir_all(self)?;
    }

    self.create(mode)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, tempfile::TempDir};

  #[test]
  fn create_with_mode() {
    let temp = TempDir::new().unwrap();

    let path = temp.path().join("test");

    path.create(0o700).unwrap();

    assert!(path.exists());
    assert!(path.is_dir());

    let metadata = fs::metadata(&path).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
  }

  #[test]
  fn recreate_with_mode() {
    let temp = TempDir::new().unwrap();

    let path = temp.path().join("test");

    path.create(0o770).unwrap();

    assert_eq!(
      fs::metadata(&path).unwrap().permissions().mode() & 0o777,
      0o770
    );

    path.recreate(0o700).unwrap();

    assert_eq!(
      fs::metadata(&path).unwrap().permissions().mode() & 0o777,
      0o700
    );
  }

  #[test]
  fn nested_create() {
    let temp = TempDir::new().unwrap();

    let path = temp.path().join("a/b/c");

    path.create(0o700).unwrap();

    assert!(path.exists());
    assert!(path.is_dir());

    assert_eq!(
      fs::metadata(&path).unwrap().permissions().mode() & 0o777,
      0o700
    );
  }
}
