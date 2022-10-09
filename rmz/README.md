# rm zippy

A zippy alternative to `rm`, a tool to remove files or directories.

## Installation

### Use prebuilt binaries

Binaries for a number of platforms are available on the
[release page](https://github.com/SUPERCILEX/fuc/releases/latest).

### Build from source

```console,ignore
$ cargo install rmz
```

## Usage

Background: https://github.com/SUPERCILEX/fuc/blob/master/README.md

Delete a file:

```console
$ rmz foo
```

Delete a directory:

```console
$ rmz dir
```

Ignore non-existent files:

```console
$ rmz -f non-existent
```

Other options:

```console
$ rmz --help
A zippy alternative to `rm`, a tool to remove files or directories

Usage: rmz [OPTIONS] <FILES>...

Arguments:
  <FILES>...
          The files and/or directories to be removed

Options:
  -f, --force
          Ignore non-existent arguments

      --no-preserve-root
          Allow deletion of `/`

  -v, --verbose...
          More output per occurrence

  -q, --quiet...
          Less output per occurrence

  -h, --help
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information

```
