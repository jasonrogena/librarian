use super::*;
use std::env;

#[test]
fn test_is_of_type() {
    let mut ok_test_cases = vec![
        (File::new("tests/files/image/bmp".to_string()), "image/bmp"),
        (
            File::new("tests/files/audio/flac".to_string()),
            "audio/flac",
        ),
        (File::new("tests/files/image/gif".to_string()), "image/gif"),
        (
            File::new("tests/files/video/mpeg".to_string()),
            "audio/mpeg",
        ),
        (
            File::new("tests/files/text/pdf".to_string()),
            "application/pdf",
        ),
        (
            File::new("tests/files/text/plain".to_string()),
            "text/plain",
        ),
        (File::new("tests/files/image/png".to_string()), "image/png"),
        (
            File::new("tests/files/image/tiff".to_string()),
            "image/tiff",
        ),
        (
            File::new("tests/files/audio/wav".to_string()),
            "application/x-riff",
        ),
        (
            File::new("tests/files/archive/x-7z-compressed".to_string()),
            "application/x-7z-compressed",
        ),
        (
            File::new("tests/files/image/x-pcx".to_string()),
            "image/vnd.zbrush.pcx",
        ),
        (
            File::new("tests/files/image/x-portable-bitmap".to_string()),
            "image/x-portable-bitmap",
        ),
        (
            File::new("tests/files/archive/x-tar".to_string()),
            "application/x-tar",
        ),
        (
            File::new("tests/files/image/x-tga".to_string()),
            "image/x-tga",
        ),
        (
            File::new("tests/files/archive/zip".to_string()),
            "application/zip",
        ),
    ];
    if env::consts::OS == "linux" {
        ok_test_cases.push((File::new("tests/files/audio/ogg".to_string()), "video/ogg"));
        ok_test_cases.push((File::new("tests/files/audio/opus".to_string()), "video/ogg"));
    } else {
        ok_test_cases.push((
            File::new("tests/files/audio/ogg".to_string()),
            "audio/x-vorbis+ogg",
        ));
        ok_test_cases.push((
            File::new("tests/files/audio/opus".to_string()),
            "audio/x-opus+ogg",
        ));
    }
    for cur_case in ok_test_cases.iter() {
        assert_eq!(cur_case.0.get_mime_type().unwrap(), cur_case.1);
    }

    let err_test_cases = [(
        File::new("tests/files/unavialabile_file".to_string()),
        "Unable to open file",
    )];
    for cur_case in err_test_cases.iter() {
        assert!(cur_case
            .0
            .get_mime_type()
            .unwrap_err()
            .get_message()
            .contains(cur_case.1));
    }
}
