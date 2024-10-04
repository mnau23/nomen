> **Note**: the following directory paths are based on macOS.

## Run

When developing, it can be useful to run the project as below.

```sh
RUST_LOG="debug" RUST_BACKTRACE=1 cargo run -- --directory "/Users/<USER>/<FOLDER>" --file-type photo
```

Optionally, add `--no-dry-run` to the previous command for disable the default execution as dry run.
