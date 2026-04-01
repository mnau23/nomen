use nomen::models::FileType;
use nomen::renamer::{DetectedKind, Renamer};
use std::path::Path;
use test_case::test_case;

/// Generate fake video files: `ffmpeg -f lavfi -i color=c=black:s=320x240:d=1 -c:v prores_ks -profile:v 0 tests/data/file.mov`.\
/// Add metadata: `ffmpeg -i tests/data/another.mp4 -metadata creation_time="2025-09-27T08:00:00Z" -codec copy tests/data/file.mp4`.
#[test_case("tests/data/video_with_creation_time.mp4", "2025-09-27" ; "with creation_time")]
#[test_case("tests/data/video_without_creation_time.mov", "NA" ; "without creation_time")]
fn test_get_date_from_video(path_str: &str, expected: &str) {
	let path = Path::new(path_str);
	let date = path.get_date_from_video().unwrap();
	assert_eq!(date, expected);
}

/// Generate fake image files: `ffmpeg -f lavfi -i color=c=white:s=1x1:d=1 tests/data/file.jpg`.\
/// Add metadata: `exiftool -DateTimeOriginal="2025:09:27 08:00:00" tests/data/file.jpg`.
#[test_case("tests/data/photo_with_exif.jpg", "2025-09-27" ; "with creation_time")]
#[test_case("tests/data/photo_without_exif.png", "NA" ; "without creation_time")]
fn test_get_date_from_photo(path_str: &str, expected: &str) {
	let path = Path::new(path_str);
	let date = path.get_date_from_photo().unwrap();
	assert_eq!(date, expected);
}

#[test_case("tests/data/photo_with_exif.jpg", true, false ; "photo file")]
#[test_case("tests/data/video_with_creation_time.mp4", false, true ; "video file")]
fn test_file_type_detection(path_str: &str, expected_photo: bool, expected_video: bool) {
	let path = Path::new(path_str);
	assert_eq!(path.is_photo(), expected_photo, "is_photo failed for {}", path_str);
	assert_eq!(path.is_video(), expected_video, "is_video failed for {}", path_str);
}

#[test_case("tests/data/photo_with_exif.jpg", FileType::Photo, 1, "2025-09-27_Photo001.jpg"; "photo file")]
fn test_generate_new_name(path_str: &str, file_type: FileType, num: usize, expected: &str) {
	let path = Path::new(path_str);
	let name = path.generate_new_name(file_type, num).unwrap();
	assert_eq!(name, expected);
}

#[test_case("tests/data/actually_photo.mp4", true, false, DetectedKind::Photo; "photo content, video extension")]
#[test_case("tests/data/actually_video.jpg", false, true, DetectedKind::Video; "video content, photo extension")]
fn test_detect_kind_ignores_wrong_extension(
	path_str: &str,
	expected_ext_video: bool,
	expected_ext_photo: bool,
	expected_kind: DetectedKind,
) {
	let path = Path::new(path_str);
	assert_eq!(path.extension_is_video(), expected_ext_video);
	assert_eq!(path.extension_is_photo(), expected_ext_photo);
	assert_eq!(path.detect_kind(), expected_kind);
}
