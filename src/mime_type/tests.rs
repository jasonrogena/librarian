use super::*;
use std::path::Path;

#[test]
fn test_is_of_type() {
    let ok_test_cases = vec![
        (File::new(Path::new("tests/files/image/bmp")), "image/bmp"),
        (File::new(Path::new("tests/files/audio/flac")), "audio/flac"),
        (File::new(Path::new("tests/files/image/gif")), "image/gif"),
        (File::new(Path::new("tests/files/video/mpeg")), "audio/mpeg"),
        (
            File::new(Path::new("tests/files/text/pdf")),
            "application/pdf",
        ),
        (File::new(Path::new("tests/files/text/plain")), "text/plain"),
        (File::new(Path::new("tests/files/image/png")), "image/png"),
        (File::new(Path::new("tests/files/image/tiff")), "image/tiff"),
        (
            File::new(Path::new("tests/files/archive/x-7z-compressed")),
            "application/x-7z-compressed",
        ),
        (
            File::new(Path::new("tests/files/image/x-pcx")),
            "image/vnd.zbrush.pcx",
        ),
        (
            File::new(Path::new("tests/files/image/x-portable-bitmap")),
            "image/x-portable-bitmap",
        ),
        (
            File::new(Path::new("tests/files/archive/x-tar")),
            "application/x-tar",
        ),
        (
            File::new(Path::new("tests/files/image/x-tga")),
            "image/x-tga",
        ),
        (
            File::new(Path::new("tests/files/archive/zip")),
            "application/zip",
        ),
    ];
    for cur_case in ok_test_cases.iter() {
        assert_eq!(cur_case.0.get_mime_type().unwrap(), cur_case.1);
    }

    let err_test_cases = [(
        File::new(Path::new("tests/files/unavialabile_file")),
        "An IO error was thrown while trying to determine the MIME type of a file",
    )];
    for cur_case in err_test_cases.iter() {
        assert!(cur_case
            .0
            .get_mime_type()
            .unwrap_err()
            .to_string()
            .contains(cur_case.1));
    }
}

#[cfg(target_os = "macos")]
#[test]
fn test_is_of_type_macos() {
    let ok_test_cases = [
        (
            File::new(Path::new("tests/files/audio/wav")),
            "application/x-riff",
        ),
        (
            File::new(Path::new("tests/files/audio/ogg")),
            "audio/x-vorbis+ogg",
        ),
        (
            File::new(Path::new("tests/files/audio/opus")),
            "audio/x-opus+ogg",
        ),
    ];
    for cur_case in ok_test_cases.iter() {
        assert_eq!(cur_case.0.get_mime_type().unwrap(), cur_case.1);
    }

    let err_test_cases = [(
        File::new(Path::new("tests/files/unavialabile_file")),
        "An IO error was thrown while trying to determine the MIME type of a file",
    )];
    for cur_case in err_test_cases.iter() {
        assert!(cur_case
            .0
            .get_mime_type()
            .unwrap_err()
            .to_string()
            .contains(cur_case.1));
    }
}

#[cfg(target_os = "linux")]
#[test]
fn test_is_of_type_linux() {
    let ok_test_cases = [
        (
            File::new(Path::new("tests/files/audio/wav")),
            "audio/vnd.wave",
        ),
        (File::new(Path::new("tests/files/audio/ogg")), "video/ogg"),
        (File::new(Path::new("tests/files/audio/opus")), "video/ogg"),
    ];
    for cur_case in ok_test_cases.iter() {
        assert_eq!(cur_case.0.get_mime_type().unwrap(), cur_case.1);
    }

    let err_test_cases = [(
        File::new(Path::new("tests/files/unavialabile_file")),
        "An IO error was thrown while trying to determine the MIME type of a file",
    )];
    for cur_case in err_test_cases.iter() {
        assert!(cur_case
            .0
            .get_mime_type()
            .unwrap_err()
            .to_string()
            .contains(cur_case.1));
    }
}
