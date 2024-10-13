> **Note**: the following directory paths are based on macOS.

## Run

When developing, it can be useful to run the project as below.

```shell
RUST_LOG="debug" RUST_BACKTRACE=1 cargo run -- --directory "/Users/<USER>/<FOLDER>"
```

Optionally:
- add `--file-type <photo|video>` for renaming only a specific file type.
- add `--no-dry-run` for disabling the default execution as dry run.
