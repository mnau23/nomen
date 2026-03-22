> **Note**: all directory paths are based on macOS.

## Installation

- First of all, check the required dependencies in the [Requirements](./requirements.md) page.
- Clone the repository locally.
- Run `cargo install --path .` to compile and install the executable: it will be installed under `/.cargo/bin`.
- Run `nomen --help` from any directory to read the usage instructions.

## Usage

It is possible to execute the program within the directory containing the files to be renamed.

A basic example for running the program is:

```shell
nomen --directory .
```

The default behaviour is:

- renaming both photos and videos.
- dry-run mode execution: this means no renaming will be applied.

Only a preview of the execution results will be shown.

### Rename only a file type

To rename only photos or videos in the specified directory, it is possible to pass another argument to the previous command.

```shell
nomen --directory . --file-type <photo|video>
```

### Disable dry run

To disable the dry run mode, add an optional argument to the previous command.

```shell
nomen --directory . --no-dry-run
```
