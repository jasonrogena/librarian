use super::*;
use std::env;

#[test]
fn test_configs() {
    let ok_test_cases = if env::consts::OS == "windows" {
        ["tests/configs/good-windows.toml".to_string()]
    } else {
        ["tests/configs/good.toml".to_string()]
    };
    for cur_case in ok_test_cases.iter() {
        Config::new(cur_case).unwrap(); // should panic if error is returned
    }

    let err_test_cases = [(
        "tests/configs/bad-missing-directories.toml".to_string(),
        "An Error was thrown while trying to parse the config as TOML",
    )];
    for cur_case in err_test_cases.iter() {
        assert!(Config::new(&cur_case.0)
            .unwrap_err()
            .to_string()
            .contains(cur_case.1));
    }
}
