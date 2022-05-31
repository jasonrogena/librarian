use super::*;

#[test]
fn test_error() {
    let err = Error::new("blah".to_string());
    assert!(format!("{}", err).contains("Error: blah"));
}
