set dotenv-load

export EDITOR := 'nvim'

alias c := check
alias f := fmt
alias t := test

default:
  just --list

build:
  cargo build

check:
  cargo clippy --all-targets --all-features

fmt:
  cargo +nightly fmt

fmt-check:
  cargo +nightly fmt -- --check

forbid:
  ./bin/forbid

test:
  cargo test

test-on-vagrant:
  vagrant up --provider=qemu
