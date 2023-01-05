# Fast Unix Commands

The FUC-ing project provides modern unix commands focused on performance.

Benchmarks are available under the [`comparisons`](comparisons/) folder.

## Commands

- [`rmz`](rmz)
- [`cpz`](cpz)

## Goals

1. Performance: if a reasonable improvement can be made, it will be.
2. Efficiency: when only negligible performance improvements are left, remaining efforts are
   focussed on minimizing wasted compute.
3. Usability: where applicable, the UX of existing commands is improved.

## Non-goals

- Portability: FUCs are primarily targeted at modern Linux installations.
- Compatibility: [coreutils](https://github.com/coreutils/coreutils) or
  its [Rust re-implementation](https://github.com/uutils/coreutils) will have the broadest and most
  stable set of options.
