use super::*;

pub trait System: std::fmt::Debug {
  fn chown(&self, path: PathBuf, uid: Option<Uid>, gid: Option<Gid>) -> Result<(), nix::Error>;
  fn getegid(&self) -> Gid;
  fn geteuid(&self) -> Uid;
  fn getgid(&self) -> Gid;
  fn getuid(&self) -> Uid;
  fn setegid(&self, gid: u32) -> Result<(), nix::Error>;
  fn umask(&self, mask: Mode) -> Mode;
}

#[derive(Debug)]
pub struct MaterialSystem;

impl System for MaterialSystem {
  fn chown(&self, path: PathBuf, uid: Option<Uid>, gid: Option<Gid>) -> Result<(), nix::Error> {
    chown(&path, uid, gid)
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

  fn setegid(&self, gid: u32) -> Result<(), nix::Error> {
    setegid(Gid::from_raw(gid))
  }

  fn umask(&self, mask: Mode) -> Mode {
    umask(mask)
  }
}
