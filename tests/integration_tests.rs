use assert_cmd::Command;
use serde_yaml::Value;
use std::path::PathBuf;

#[test]
fn binary_correctly_reads_and_prints_out_example_configuration() {
    let mut binary = Command::cargo_bin("fake-ci").unwrap();

    let mut path_to_configuration = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path_to_configuration.push("tests/configurations/example-gitlab-ci.yml");
    let path_to_configuration = path_to_configuration.to_str().unwrap();

    let assert = binary
        .arg("--configuration-file")
        .arg(path_to_configuration)
        .arg("print")
        .assert()
        .success();

    let output = assert.get_output();
    let content = std::str::from_utf8(&output.stdout).unwrap();
    let output_as_yaml: Value = serde_yaml::from_str(content).unwrap();

    assert!(matches!(output_as_yaml, Value::Mapping { .. }));
}
