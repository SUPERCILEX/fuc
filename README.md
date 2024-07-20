# Fast Unix Commands

[![cpz crates.io](https://img.shields.io/crates/v/cpz?label=cpz%20crates.io)](https://crates.io/crates/cpz)
[![rmz crates.io](https://img.shields.io/crates/v/rmz?label=rmz%20crates.io)](https://crates.io/crates/rmz)
[![Packaging status](https://repology.org/badge/tiny-repos/fuc.svg)](https://repology.org/project/fuc/badges)

The FUC-ing project provides modern unix commands focused on performance:

- [`cpz`](cpz): a zippy alternative to [`cp`](https://man7.org/linux/man-pages/man1/cp.1.html)
- [`rmz`](rmz): a zippy alternative to [`rm`](https://man7.org/linux/man-pages/man1/rm.1.html)

Benchmarks are available under the [`comparisons`](comparisons) folder and a brief technical
overview is available at https://alexsaveau.dev/blog/fuc.

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
