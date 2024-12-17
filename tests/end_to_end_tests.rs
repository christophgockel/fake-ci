use assert_cmd::assert::Assert;
use assert_cmd::Command;
use predicates::str::contains;
use std::ffi::OsStr;
use std::path::PathBuf;

#[test]
#[cfg_attr(not(feature = "docker_tests"), ignore)]
fn artifacts_from_one_job_are_available_in_subsequent_jobs() {
    // The `build` job creates a file with content "build time content".
    assert_command("fake-ci", "run", "build").success();
    // That file is to be expected to exist in the `test` job.
    assert_command("fake-ci", "run", "test")
        .success()
        .stdout(contains("build time content"));
}

fn assert_command<S: AsRef<OsStr>>(binary: &str, command: S, argument: S) -> Assert {
    let mut path_to_configuration = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path_to_configuration.push("tests/configurations/end-to-end-gitlab-ci.yml");
    let path_to_configuration = path_to_configuration.to_str().unwrap();

    Command::cargo_bin(binary)
        .unwrap()
        .arg("--configuration-file")
        .arg(path_to_configuration)
        .arg(command)
        .arg(argument)
        .assert()
}
