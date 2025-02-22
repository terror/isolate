#![cfg(feature = "integration")]

use {
  assert_matches::assert_matches,
  isolate::{Config, Environment, Error, Sandbox},
  nix::unistd::{geteuid, seteuid, Uid},
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
fn sandbox_id_out_of_range() {
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
