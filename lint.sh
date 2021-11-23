#!/usr/bin/env bash

cargo +nightly fmt
cargo +nightly clippy --fix --allow-dirty
cargo +nightly fix --allow-dirty
