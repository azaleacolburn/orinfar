use std::fs;

use anyhow::Result;

const TEST_DIR: &str = "./tests/";

struct Test {
    input: String,
    output: String,
}

#[test]
fn integration_testing() -> Result<()> {
    let entries = std::fs::read_dir(TEST_DIR)?;
    let tests = entries
        .filter_map(|test| test.ok())
        .filter_map(|test| test.file_name().into_string().ok())
        .filter(|test| {
            let extension = test.split(".").last().unwrap_or("").to_string();

            extension == "in"
        })
        .map(|input_file_name| {
            let input =
                std::fs::read_to_string(format!("{}{}", TEST_DIR, &input_file_name)).unwrap();

            let split: Vec<&str> = input_file_name.split(".").collect();
            assert!(!split.is_empty());
            let output_file_name = format!("{}.out", &split[0..split.len() - 1].join(""));

            let output = fs::read_to_string(format!("{}{}", TEST_DIR, output_file_name)).unwrap();

            Test { input, output }
        });

    // TODO Create mock system
    for test in tests {}

    Ok(())
}
