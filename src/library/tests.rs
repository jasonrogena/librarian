use super::*;
use crate::config;
use std::collections::HashMap;
use std::env::consts::OS;
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_process() {
    let conf = if OS == "windows" {
        config::Config::new(&"tests\\configs\\good-windows.toml".to_string())
    } else {
        config::Config::new(&"tests/configs/good.toml".to_string())
    };
    let mut test_cases: HashMap<String, (Vec<String>, String)> = HashMap::new();
    test_cases.insert(
        "audio".to_string(),
        (
            vec!["audio_tests_files_audio_flac".to_string()],
            "is an audio file. The file's MIME type is".to_string(),
        ),
    );
    test_cases.insert(
        "videos".to_string(),
        (
            vec!["video_tests_files_video_mpeg".to_string()],
            "s a video. The file's MIME type is".to_string(),
        ),
    );

    for (cur_lib_key, cur_lib_val) in conf.unwrap().libraries {
        if !test_cases.contains_key(&cur_lib_key) {
            continue;
        }
        let skip_running_commands = false;
        let lib = Library::new(&cur_lib_val, &skip_running_commands);
        assert_eq!(
            lib.process(None).unwrap(),
            test_cases.get(&cur_lib_key).unwrap().0.len() as u64
        );

        for cur_path in test_cases.get(&cur_lib_key).unwrap().0.iter() {
            let path_buf: PathBuf = ["tests", "tmp", cur_path].iter().collect();
            let contents = fs::read_to_string(path_buf.as_path()).unwrap();
            assert!(contents.contains(test_cases.get(&cur_lib_key).unwrap().1.as_str()));
        }
    }
}

#[test]
fn test_process_single_file() {
    let conf = if OS == "windows" {
        config::Config::new(&"tests\\configs\\good-windows.toml".to_string())
    } else {
        config::Config::new(&"tests/configs/good.toml".to_string())
    };
    let mut test_cases: HashMap<String, (Vec<(String, String)>, String)> = HashMap::new();
    if OS == "windows" {
        test_cases.insert(
            "books".to_string(),
            (
                vec![
                    (
                        "tests\\files\\text\\pdf".to_string(),
                        "books_tests_files_text_pdf".to_string(),
                    ),
                    (
                        "tests\\files\\text\\plain".to_string(),
                        "books_tests_files_text_plain".to_string(),
                    ),
                ],
                "is a book. The file's MIME type is".to_string(),
            ),
        );
    } else {
        test_cases.insert(
            "books".to_string(),
            (
                vec![
                    (
                        "tests/files/text/pdf".to_string(),
                        "books_tests_files_text_pdf".to_string(),
                    ),
                    (
                        "tests/files/text/plain".to_string(),
                        "books_tests_files_text_plain".to_string(),
                    ),
                ],
                "is a book. The file's MIME type is".to_string(),
            ),
        );
    }

    let skip_running_commands = false;
    for (cur_lib_key, cur_lib_val) in conf.unwrap().libraries {
        if !test_cases.contains_key(&cur_lib_key) {
            continue;
        }
        let lib = Library::new(&cur_lib_val, &skip_running_commands);

        for cur_file in test_cases.get(&cur_lib_key).unwrap().0.iter() {
            let cur_test_path = Path::new(cur_file.0.as_str());
            assert_eq!(lib.process(Some(cur_test_path)).unwrap(), 1u64);
            let cur_generated_path: PathBuf =
                ["tests", "tmp", cur_file.1.as_str()].iter().collect();
            let contents = fs::read_to_string(cur_generated_path.as_path()).unwrap();
            assert!(contents.contains(test_cases.get(&cur_lib_key).unwrap().1.as_str()));
        }
    }
}

#[test]
fn test_contains_path() {
    let conf = if OS == "windows" {
        config::Config::new(&"tests\\configs\\good-windows.toml".to_string()).unwrap()
    } else {
        config::Config::new(&"tests/configs/good.toml".to_string()).unwrap()
    };
    let cur_dir = current_dir().unwrap();
    let skip_running_commands = false;
    let audio_lib = Library::new(&conf.libraries["audio"], &skip_running_commands);
    assert!(!audio_lib.contains_path(Path::new("")));
    if OS == "windows" {
        assert!(audio_lib.contains_path(Path::new("tests\\files\\audio\\flac")));
        assert!(!audio_lib.contains_path(Path::new("tests\\files\\audio\\fl")));
    } else {
        assert!(audio_lib.contains_path(Path::new("tests/files/audio/flac")));
        assert!(!audio_lib.contains_path(Path::new("tests/files/audio/fl")));
        assert!(audio_lib.contains_path(Path::new(
            format!(
                "{}/tests/files/audio/flac",
                cur_dir.as_os_str().to_str().unwrap()
            )
            .as_str()
        )));
        assert!(!audio_lib.contains_path(Path::new(
            format!(
                "{}/tests/files/audio/fl",
                cur_dir.as_os_str().to_str().unwrap()
            )
            .as_str()
        )));
    }
}
