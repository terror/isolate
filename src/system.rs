use super::*;

pub trait System: std::fmt::Debug {
  fn chown(&self, path: &Utf8Path, uid: Option<Uid>, gid: Option<Gid>) -> Result;
  fn create_directory_with_mode(&self, path: &Utf8Path, mode: u32) -> Result;
  fn getegid(&self) -> Gid;
  fn geteuid(&self) -> Uid;
  fn getgid(&self) -> Gid;
  fn getuid(&self) -> Uid;
  fn recreate_directory_with_mode(&self, path: &Utf8Path, mode: u32) -> Result;
  fn setegid(&self, gid: u32) -> Result;
  fn umask(&self, mask: Mode) -> Mode;
}

#[derive(Debug)]
pub struct MaterialSystem;

impl System for MaterialSystem {
  fn chown(&self, path: &Utf8Path, uid: Option<Uid>, gid: Option<Gid>) -> Result {
    chown(&PathBuf::from(path), uid, gid)
      .map_err(|error| Error::Permission(format!("failed to chown `{}`: {}", path, error)))
  }

  fn create_directory_with_mode(&self, path: &Utf8Path, mode: u32) -> Result {
    fs::create_dir_all(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
  }

  fn getegid(&self) -> Gid {
    getegid()
  }

  fn geteuid(&self) -> Uid {
    geteuid()
  }

  fn getgid(&self) -> Gid {
    getgid()
  }

  fn getuid(&self) -> Uid {
    getuid()
  }

  fn recreate_directory_with_mode(&self, path: &Utf8Path, mode: u32) -> Result {
    if path.exists() {
      fs::remove_dir_all(path)?;
    }

    self.create_directory_with_mode(path, mode)
  }

  fn setegid(&self, gid: u32) -> Result {
    setegid(Gid::from_raw(gid))
      .map_err(|error| Error::Permission(format!("failed to setegid: {}", error)))
  }

  fn umask(&self, mask: Mode) -> Mode {
    umask(mask)
  }
}
