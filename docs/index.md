> **Note**: the following directory paths are based on macOS.

## Installation

- Clone the repository locally.
- Run `cargo install --path .` to compile and install the executable.
  - It will be installed under `/.cargo/bin`.
- Run `nomen --help` from any directory to read the usage instructions.

## Usage

It is possible to execute the command within the directory with the files to be renamed.  An example of use is:

```sh
nomen --directory . --file-type photo
```

In this case, since `dry-run` is not specified, it defaults to `true`, so no renaming will be applied. Otherwise, run:

```sh
nomen --directory . --file-type photo --dry-run false
```

If interested in looking at the logs, prepend `RUST_LOG=debug` before the `nomen` command.
