use super::*;

pub trait System {
  fn getegid(&self) -> Gid;
  fn geteuid(&self) -> Uid;
  fn getgid(&self) -> Gid;
  fn getuid(&self) -> Uid;
  fn setegid(&self, gid: u32) -> Result<(), nix::Error>;
  fn umask(&self, mask: Mode) -> Mode;
}

pub struct MaterialSystem;

impl System for MaterialSystem {
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
