# Requirements

## Build & Runtime

- Rust toolchain (`cargo`) with **Rust v1.85.0** or above installed
- FFmpeg libraries, since it's used by `ffmpeg-next`

  ```shell
  # macOS
  brew install ffmpeg
  ```

## Test

Currently tests ship media files under `tests/data/`, so `cargo test` does **not** require extra tools. \
If you want to regenerate those fixtures locally, you'll need:

- `ffmpeg`: already installed for the build/runtime
- `exiftool`: to write the `DateTimeOriginal` EXIF tags

  ```shell
  # macOS
  brew install exiftool
  ```

## Dev tooling

- `cargo-make`: to run `cargo make nomen`
- `mdbook`: to build the documentation under `docs/`

Both can be installed with:

```shell
cargo install cargo-make mdbook
```
