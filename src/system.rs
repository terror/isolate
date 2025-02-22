use super::*;

pub trait System: std::fmt::Debug {
  fn chown(&self, path: &Path, uid: Option<Uid>, gid: Option<Gid>) -> Result<(), nix::Error>;
  fn create_directory(&self, path: &Path, mode: u32) -> Result;
  fn getegid(&self) -> Gid;
  fn geteuid(&self) -> Uid;
  fn getgid(&self) -> Gid;
  fn getuid(&self) -> Uid;
  fn recreate_directory(&self, path: &Path, mode: u32) -> Result;
  fn setegid(&self, gid: u32) -> Result<(), nix::Error>;
  fn umask(&self, mask: Mode) -> Mode;
}

#[derive(Debug)]
pub struct MaterialSystem;

impl System for MaterialSystem {
  fn chown(&self, path: &Path, uid: Option<Uid>, gid: Option<Gid>) -> Result<(), nix::Error> {
    chown(path, uid, gid)
  }

  fn create_directory(&self, path: &Path, mode: u32) -> Result {
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

  fn recreate_directory(&self, path: &Path, mode: u32) -> Result {
    if path.exists() {
      fs::remove_dir_all(path)?;
    }

    self.create_directory(path, mode)
  }

  fn setegid(&self, gid: u32) -> Result<(), nix::Error> {
    setegid(Gid::from_raw(gid))
  }

  fn umask(&self, mask: Mode) -> Mode {
    umask(mask)
  }
}
