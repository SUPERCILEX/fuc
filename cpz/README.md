# cp zippy

[![Crates.io](https://img.shields.io/crates/v/cpz?style=flat-square)](https://crates.io/crates/cpz)

A zippy alternative to `cp`, a tool to copy files and directories.

## Installation

### Use prebuilt binaries

Binaries for a number of platforms are available on the
[release page](https://github.com/SUPERCILEX/fuc/releases/latest).

### Build from source

```console,ignore
$ cargo install cpz
```

> To install cargo, follow
> [these instructions](https://doc.rust-lang.org/cargo/getting-started/installation.html).

## Usage

Background: https://github.com/SUPERCILEX/fuc/blob/master/README.md

Copy a file:

```console
$ cpz from to
```

Copy a directory:

```console
$ cpz from_dir to_dir
```

Overwrite existing files:

```console
$ cpz -f from existing
```

Flip the argument order (for better composability with other commands for example):

```console
$ cpz -t to_first from
```

Force the source files to be copied into the destination by making the path look like a directory:

```console
$ cpz from dest/
```

More details:

```console
$ cpz --help
A zippy alternative to `cp`, a tool to copy files and directories

Usage: cpz[EXE] [OPTIONS] <FROM>... <TO>

Arguments:
  <FROM>...
          The file(s) or directory(ies) to be copied
          
          If multiple files are specified, they will be copied into the target destination rather
          than to it. The same is true of directory names (`foo/`, `.`, `..`): that is, `cpz a b/`
          places `a` inside `b` as opposed to `cpz a b` which makes `b` become `a`.

  <TO>
          The copy destination

Options:
  -f, --force
          Overwrite existing files

  -t, --reverse-args
          Reverse the argument order so that it becomes `cpz <TO> <FROM>...`

  -h, --help
          Print help (use `-h` for a summary)

  -V, --version
          Print version

```
