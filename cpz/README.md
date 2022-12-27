# cp zippy

A zippy alternative to `cp`, a tool to remove files or directories.

## Installation

### Use prebuilt binaries

Binaries for a number of platforms are available on the
[release page](https://github.com/SUPERCILEX/fuc/releases/latest).

### Build from source

```console,ignore
$ cargo install cpz
```

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

Other options:

```console
$ cpz --help
A zippy alternative to `cp`, a tool to copy files and directories

Usage: cpz[EXE] [OPTIONS] <FROM> <TO>

Arguments:
  <FROM>
          The file or directory to be copied

  <TO>
          The copy destination

Options:
  -f, --force
          Overwrite existing files

  -h, --help
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information

```
