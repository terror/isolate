set dotenv-load

export EDITOR := 'nvim'

alias f := fmt
alias t := test

default:
  just --list

fmt:
  cargo +nightly fmt

forbid:
  ./bin/forbid

test:
  cargo test
