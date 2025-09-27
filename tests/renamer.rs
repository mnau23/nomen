use std::path::Path;
use nomen::renamer::Renamer;
use test_case::test_case;

/// Generate fake video files with ffmpeg:
/// `ffmpeg -f lavfi -i color=c=black:s=320x240:d=1 -c:v prores_ks -profile:v 0 tests/data/file.mov`
/// TODO: fix docs
#[test_case("tests/data/video_with_creation_time.mp4", "2025-09-27" ; "with creation_time")]
#[test_case("tests/data/video_without_creation_time.mov", "NA" ; "without creation_time")]
fn test_get_date_from_video(path_str: &str, expected: &str) {
    let path = Path::new(path_str);
    let date = path.get_date_from_video().unwrap();
    assert_eq!(date, expected);
}

/// Generate fake image files
/// ffmpeg -f lavfi -i color=c=white:s=1x1:d=1 tests/data/photo_with_exif.jpg
/// exiftool -DateTimeOriginal="2025:09:27 08:00:00" tests/data/photo_with_exif.jpg
/// TODO: fix docs
#[test_case("tests/data/photo_with_exif.jpg", "2025-09-27" ; "with creation_time")]
#[test_case("tests/data/photo_without_exif.png", "NA" ; "without creation_time")]
fn test_get_date_from_photo(path_str: &str, expected: &str) {
    let path = Path::new(path_str);
    let date = path.get_date_from_photo().unwrap();
    assert_eq!(date, expected);
}
