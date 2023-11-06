#[test]
fn test_generated_struct() {
    use xtask::generated_struct::Generated_structQueryResult;

    let json_string = r#"
        [
            1,
            1,
            1,
            "12345678901",
            1.2345,
            1.2345678901234567,
            true,
            "2023-01-01",
            "2023-01-01 01:02:03.456",
            "2023-01-01 01:02:03.456 UTC",
            "01:02:03.456",
            "a",
            "a variable-length string",
            "REVBREJFRUY=",
            "{ \"key\": \"value\" }",
            [1, 2, 3],
            {
                "key1": 1,
                "key2": 2
            },
            "123e4567-e89b-12d3-a456-426614174000"
        ]
        "#;
    let res: Generated_structQueryResult = serde_json::from_str(json_string).unwrap();
}
