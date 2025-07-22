# Contributing

> **Note**: all directory paths are based on macOS.

## Run

When developing, it can be useful to run the project as below.

```shell
RUST_BACKTRACE=1 \
RUST_LOG="debug" \                                                                                               INT ✘ 
cargo run -- --directory "/Users/<USER>/<FOLDER>"
```

Optionally:
- add `--file-type <photo|video>` for renaming only a specific file type.
- add `--no-dry-run` for disabling the default execution as dry run.

## Test

Some unit tests are provided in the project, run them easily with `cargo test`.
