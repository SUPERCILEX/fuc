# Fast Unix Commands

The FUC-ing project provides modern unix commands focused on performance.

Benchmarks are available under the [`comparisons`](comparisons/) folder.

[![cpz Crates.io](https://img.shields.io/crates/v/cpz.svg)](https://crates.io/crates/cpz)
[![rmz Crates.io](https://img.shields.io/crates/v/rmz.svg)](https://crates.io/crates/rmz)
[![Packaging status](https://repology.org/badge/tiny-repos/fuc.svg)](https://repology.org/project/fuc/badges)
[![License](http://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/SUPERCILEX/fuc/blob/master/LICENSE)
[![dependency status](https://deps.rs/repo/github/SUPERCILEX/fuc/status.svg)](https://deps.rs/repo/github/SUPERCILEX/fuc)
![MSRV](https://img.shields.io/badge/MSRV-1.68.0-brightgreen)

## Commands

- [`rmz`](rmz)
- [`cpz`](cpz)

## Goals

1. Performance: if a reasonable improvement can be made, it will be.
2. Efficiency: when only negligible performance improvements are left, remaining efforts are
   focussed on minimizing wasted compute.
3. Usability: where applicable, the UX of existing commands is improved.

## Non-goals

- Portability: FUCs are primarily targeted at modern Linux installations. Support for other
  platforms is provided on a best-efforts basis.
- Compatibility: [coreutils](https://github.com/coreutils/coreutils) or
  its [Rust re-implementation](https://github.com/uutils/coreutils) will have the broadest and most
  stable set of options.
  
## Installing with cargo

    cargo install cpz
    cargo install rmz

## Building from source

As of v1.1.7 ths project builds with rust stable >= 1.68.

    git clone [...]
    cargo build --all

Running tests still requires rust nightly:

    cargo test

To install the loccally built binaries:

    cargo install --path cpz
    cargo install --path mpv
