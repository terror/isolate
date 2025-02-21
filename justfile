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

dev-deps:
  brew install --cask vagrant
  vagrant plugin install vagrant-qemu
  brew install qemu

fmt:
  cargo +nightly fmt

fmt-check:
  cargo +nightly fmt -- --check

forbid:
  ./bin/forbid

test *args:
  cargo test {{args}}

test-on-vagrant:
  ./bin/integration
