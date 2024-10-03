> **Note**: the following directory paths are based on macOS.

## Installation

- Clone the repository locally.
- Run `cargo install --path .` to compile and install the executable: it will be installed under `/.cargo/bin`.
- Run `nomen --help` from any directory to read the usage instructions.

## Usage

It is possible to execute the command within the directory containing the files to be renamed. An example of use is:

```sh
nomen --directory . --file-type photo
```

The default behaviour is the execution of the program in a dry run mode: this means no renaming will be applied.
Only a preview of the execution results will be shown.

To disable the dry run mode, add to the previous command an option as below.

```sh
nomen --directory . --file-type photo --no-dry-run
```
