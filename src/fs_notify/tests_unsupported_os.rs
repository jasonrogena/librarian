use super::*;
use std::collections::HashSet;
use std::sync::mpsc::channel;

#[test]
fn test_unsupported_os() {
    let paths: HashSet<String> = HashSet::new();
    let (on_event_sender, _) = channel();
    match Notify::new(&None, &paths, on_event_sender) {
        Ok(_) => panic!(),
        Err(e) => assert_eq!(
            e.get_message(),
            "Directory watching is currently not supported in this OS"
        ),
    }
}
