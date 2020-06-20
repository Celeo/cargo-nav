default:
  cargo check --all
  cargo build --all

check_cargo_subcommand:
  PATH=$PATH:./target/debug cargo nav --help

alias ccs := check_cargo_subcommand
