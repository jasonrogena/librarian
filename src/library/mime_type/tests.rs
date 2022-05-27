use super::*;

#[test]
fn test_is_of_type() {
    let ok_test_cases = [
        (File::new("tests/files/bmp".to_string()), "image/bmp"),
        (File::new("tests/files/flac".to_string()), "audio/flac"),
        (File::new("tests/files/gif".to_string()), "image/gif"),
        (File::new("tests/files/mpeg".to_string()), "audio/mpeg"),
        (
            File::new("tests/files/ogg".to_string()),
            "audio/x-vorbis+ogg",
        ),
        (
            File::new("tests/files/opus".to_string()),
            "audio/x-opus+ogg",
        ),
        (File::new("tests/files/pdf".to_string()), "application/pdf"),
        (File::new("tests/files/plain".to_string()), "text/plain"),
        (File::new("tests/files/png".to_string()), "image/png"),
        (File::new("tests/files/tiff".to_string()), "image/tiff"),
        (
            File::new("tests/files/wav".to_string()),
            "application/x-riff",
        ),
        (
            File::new("tests/files/x-7z-compressed".to_string()),
            "application/x-7z-compressed",
        ),
        (
            File::new("tests/files/x-pcx".to_string()),
            "image/vnd.zbrush.pcx",
        ),
        (
            File::new("tests/files/x-portable-bitmap".to_string()),
            "image/x-portable-bitmap",
        ),
        (
            File::new("tests/files/x-tar".to_string()),
            "application/x-tar",
        ),
        (File::new("tests/files/x-tga".to_string()), "image/x-tga"),
        (File::new("tests/files/zip".to_string()), "application/zip"),
    ];
    for cur_case in ok_test_cases.iter() {
        assert_eq!(cur_case.0.get_mime_type().unwrap(), cur_case.1);
    }

    let err_test_cases = [(
        File::new("tests/files/unavialabile_file".to_string()),
        "Unable to open file",
    )];
    for cur_case in err_test_cases.iter() {
        assert_eq!(
            cur_case
                .0
                .get_mime_type()
                .unwrap_err()
                .get_message()
                .contains(cur_case.1),
            true
        );
    }
}
