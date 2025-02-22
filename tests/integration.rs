#![cfg(feature = "integration")]

use {
  assert_matches::assert_matches,
  isolate::{Config, Environment, Error, Sandbox},
  nix::unistd::{geteuid, seteuid, Uid},
  std::{fs, os::unix::fs::PermissionsExt, path::PathBuf},
  tempfile::TempDir,
};

#[test]
fn sandbox_construction_as_non_root() {
  let original_euid = geteuid();

  seteuid(Uid::from_raw(1000)).unwrap();

  let environment = Environment {
    num_sandboxes: 10,
    ..Default::default()
  };

  let config = Config {
    sandbox_id: Some(0),
    ..Default::default()
  };

  let result = Sandbox::try_from((config, &environment));

  assert_matches!(result, Err(Error::NotRoot));

  seteuid(original_euid).unwrap();

  assert_eq!(geteuid(), original_euid);
}

#[test]
fn sandbox_construction_id_out_of_range() {
  let environment = Environment {
    num_sandboxes: 10,
    ..Default::default()
  };

  let config = Config {
    sandbox_id: Some(11),
    ..Default::default()
  };

  let result = Sandbox::try_from((config, &environment));

  assert_matches!(
    result,
    Err(Error::Config(message)) if message.contains("sandbox id out of range")
  );
}

#[test]
fn sandbox_initialization_creates_sandbox_directories() {
  let temp_dir = TempDir::new().unwrap();

  let ancestor_permissions: Vec<(PathBuf, fs::Permissions)> = temp_dir
    .path()
    .ancestors()
    .map(|path| {
      (
        path.to_path_buf(),
        fs::metadata(path).unwrap().permissions(),
      )
    })
    .collect();

  // n.b. It is mainly `/tmp` that needs to be set accordingly.
  for ancestor in temp_dir.path().ancestors() {
    fs::set_permissions(ancestor, fs::Permissions::from_mode(0o700)).unwrap();
  }

  let sandbox_root = temp_dir.path().join("sandbox_root");

  let environment = Environment {
    sandbox_root: sandbox_root.clone(),
    ..Default::default()
  };

  let config = Config {
    sandbox_id: Some(0),
    ..Default::default()
  };

  let sandbox = Sandbox::try_from((config, &environment)).unwrap();

  sandbox.initialize().unwrap();

  assert!(sandbox_root.exists());

  assert!(sandbox.directory().exists());

  let metadata = fs::metadata(sandbox.directory())
    .unwrap()
    .permissions()
    .mode();

  assert_eq!(metadata & 0o777, 0o700);

  assert!(sandbox.directory().join("box").exists());

  let metadata = fs::metadata(sandbox.directory().join("box"))
    .unwrap()
    .permissions()
    .mode();

  assert_eq!(metadata & 0o777, 0o700);

  for (path, permissions) in ancestor_permissions {
    fs::set_permissions(path, permissions).unwrap();
  }
}

#[test]
fn sandbox_initialization_fails_on_bad_ancestor_permissions() {
  let temp_dir = TempDir::new().unwrap();

  let sandbox_root = temp_dir.path().join("sandbox_root");

  let environment = Environment {
    sandbox_root: sandbox_root.clone(),
    ..Default::default()
  };

  let config = Config {
    sandbox_id: Some(0),
    ..Default::default()
  };

  let sandbox = Sandbox::try_from((config, &environment)).unwrap();

  assert!(matches!(
    sandbox.initialize(),
    Err(Error::Permission(msg)) if msg.contains("must be writable only by root")
  ));
}
