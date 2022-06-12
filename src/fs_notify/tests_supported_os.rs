use super::*;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::path::MAIN_SEPARATOR;
use std::sync::mpsc::channel;
use std::thread;
use std::time;

#[test]
fn test_watch() {
    let mut test_dir = PathBuf::from("tests");
    test_dir.push("tmp");
    test_dir.push("fs_notify_watch");
    let mut test_sub_dir = test_dir.clone();
    test_sub_dir.push("sub_dir");
    fs::create_dir_all(test_sub_dir.clone()).unwrap();
    let test_str = format!("{:?}", time::SystemTime::now());
    let mut paths: HashSet<String> = HashSet::new();
    paths.insert(test_dir.as_os_str().to_str().unwrap().to_string());
    paths.insert(test_sub_dir.as_os_str().to_str().unwrap().to_string());
    let (on_event_sender, on_event_receiver) = channel();
    let (mut notify_obj, unwatch_sender) = Notify::new(&None, &paths, on_event_sender).unwrap();

    thread::spawn(move || {
        notify_obj.watch();
    });
    let test_str_clone = test_str.to_string();
    let (run_tests_sender, run_tests_receiver) = channel();
    thread::spawn(move || {
        if let Ok(received_path_1) = on_event_receiver.recv_timeout(time::Duration::from_secs(10)) {
            println!("**{}**", received_path_1);
            let p = Path::new(received_path_1.as_str())
                .as_os_str()
                .to_str()
                .unwrap();
            let mut woot_file =
                fs::File::create(generate_test_output_filename("watch", p)).unwrap();
            write!(woot_file, "{}", test_str_clone).unwrap();
        }
        Notify::unwatch(&unwatch_sender);
        run_tests_sender.send(true).unwrap();
    });

    let mut test_file_path = test_sub_dir.clone();
    test_file_path.push("test_file");
    let mut test_file = fs::File::create(test_file_path.clone()).unwrap();
    write!(test_file, "foo").unwrap();

    run_tests_receiver.recv().unwrap();
    let file_data_1 = fs::read_to_string(generate_test_output_filename(
        "watch",
        test_file_path.as_os_str().to_str().unwrap(),
    ))
    .unwrap();
    assert_eq!(file_data_1, test_str.as_str());
}

#[test]
fn test_notify_ttl() {
    let mut test_dir = PathBuf::from("tests");
    test_dir.push("tmp");
    test_dir.push("fs_notify_ttl");
    let mut test_sub_dir = test_dir.clone();
    test_sub_dir.push("sub_dir");
    fs::create_dir_all(test_sub_dir.clone()).unwrap();
    let test_str = format!("{:?}", time::SystemTime::now());
    let mut paths: HashSet<String> = HashSet::new();
    paths.insert(test_dir.as_os_str().to_str().unwrap().to_string());
    paths.insert(test_sub_dir.as_os_str().to_str().unwrap().to_string());
    let (on_event_sender, on_event_receiver) = channel();
    let (mut notify_obj, unwatch_sender) = Notify::new(
        &Some(FsWatch {
            notification_ttl: Some(60),
        }),
        &paths,
        on_event_sender,
    )
    .unwrap();

    thread::spawn(move || {
        notify_obj.watch();
    });
    let test_str_clone = test_str.to_string();
    let (run_tests_sender, run_tests_receiver) = channel();
    thread::spawn(move || {
        if let Ok(received_path_1) = on_event_receiver.recv_timeout(time::Duration::from_secs(10)) {
            println!("**{}**", received_path_1);
            let p = Path::new(received_path_1.as_str())
                .as_os_str()
                .to_str()
                .unwrap();
            let mut woot_file = fs::File::create(generate_test_output_filename("ttl", p)).unwrap();
            write!(woot_file, "{}", test_str_clone).unwrap();

            // We expect the notification to be sent only once, if it's sent more times, write a
            // 'nope' in the file
            if on_event_receiver
                .recv_timeout(time::Duration::from_secs(10))
                .is_ok()
            {
                write!(woot_file, "nope").unwrap();
            }
        }
        Notify::unwatch(&unwatch_sender);
        run_tests_sender.send(true).unwrap();
    });

    let mut test_file_path = test_sub_dir.clone();
    test_file_path.push("test_file");
    let mut test_file = fs::File::create(test_file_path.clone()).unwrap();
    write!(test_file, "foo").unwrap();
    write!(test_file, "bar").unwrap();

    run_tests_receiver.recv().unwrap();
    let file_data_1 = fs::read_to_string(generate_test_output_filename(
        "ttl",
        test_file_path.as_os_str().to_str().unwrap(),
    ))
    .unwrap();
    assert_eq!(file_data_1, test_str.as_str());
}

fn generate_test_output_filename(prefix: &str, path: &str) -> String {
    let p = fs::canonicalize(PathBuf::from(path)).unwrap();
    format!(
        "tests{MAIN_SEPARATOR}tmp{MAIN_SEPARATOR}fs_notify_{}_{}",
        prefix,
        p.as_os_str().to_str().unwrap().replace(MAIN_SEPARATOR, "-")
    )
}
