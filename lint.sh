#!/usr/bin/env bash

cargo fmt
cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged --\
  -W clippy::all \
  -W clippy::float_cmp_const \
  -W clippy::empty_structs_with_brackets \
  -W clippy::pedantic \
  -W clippy::nursery \
  -W clippy::cargo
