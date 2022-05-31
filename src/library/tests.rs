use super::*;
use crate::config;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_process() {
    let conf = if env::consts::OS == "windows" {
        config::Config::new("tests\\configs\\good-windows.toml")
    } else {
        config::Config::new("tests/configs/good.toml")
    };
    let mut test_cases: HashMap<String, (Vec<String>, String)> = HashMap::new();
    if env::consts::OS == "linux" {
        test_cases.insert(
            "audio".to_string(),
            (
                vec![
                    "audio_tests_files_audio_flac".to_string(),
                    "audio_tests_files_video_mpeg".to_string(),
                ],
                "is an audio file. The file's MIME type is".to_string(),
            ),
        );
    } else {
        test_cases.insert(
            "audio".to_string(),
            (
                vec![
                    "audio_tests_files_audio_flac".to_string(),
                    "audio_tests_files_audio_ogg".to_string(),
                    "audio_tests_files_audio_opus".to_string(),
                    "audio_tests_files_video_mpeg".to_string(),
                ],
                "is an audio file. The file's MIME type is".to_string(),
            ),
        );
    }
    test_cases.insert(
        "videos".to_string(),
        (
            vec!["video_tests_files_video_mpeg".to_string()],
            "s a video. The file's MIME type is".to_string(),
        ),
    );
    test_cases.insert(
        "books".to_string(),
        (
            vec![
                "books_tests_files_text_pdf".to_string(),
                "books_tests_files_text_plain".to_string(),
            ],
            "is a book. The file's MIME type is".to_string(),
        ),
    );

    for (cur_lib_key, cur_lib_val) in conf.unwrap().libraries {
        let lib = Library::new(cur_lib_val);
        assert_eq!(
            lib.process().unwrap(),
            test_cases.get(&cur_lib_key).unwrap().0.len() as u64
        );

        for cur_path in test_cases.get(&cur_lib_key).unwrap().0.iter() {
            let path_buf: PathBuf = ["tests", "tmp", cur_path].iter().collect();
            let contents = fs::read_to_string(path_buf.as_path()).unwrap();
            assert!(contents.contains(test_cases.get(&cur_lib_key).unwrap().1.as_str()));
        }
    }
}
