use super::*;

#[test]
fn test_render() {
    let ok_test_cases: Vec<(&str, collections::HashMap<&str, &str>, &str)> = vec![
        (
            "Test {{ var_1 }} before {{ var_2 }}.",
            [("var_1", "hello"), ("var_2", "world")]
                .iter()
                .cloned()
                .collect(),
            "Test hello before world.",
        ),
        (
            "Just {{ var_1 }}.",
            [("var_1", "hello"), ("var_2", "world")]
                .iter()
                .cloned()
                .collect(),
            "Just hello.",
        ),
    ];

    for cur_test_case in ok_test_cases.iter() {
        let cur_template = Template::new(cur_test_case.0.to_string()).unwrap();
        assert_eq!(
            cur_template.render(&cur_test_case.1).unwrap(),
            cur_test_case.2
        );
    }

    let err_test_cases: Vec<(&str, collections::HashMap<&str, &str>, &str)> = vec![(
        "Test {{ var_1 }} before {{ var_2 }}.",
        [("var_1", "hello")].iter().cloned().collect(),
        "A Tera templating error occurred",
    )];

    for cur_test_case in err_test_cases.iter() {
        let cur_template = Template::new(cur_test_case.0.to_string()).unwrap();
        assert!(cur_template
            .render(&cur_test_case.1)
            .unwrap_err()
            .to_string()
            .contains(cur_test_case.2));
    }
}
