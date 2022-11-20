use duct::cmd;
use std::io::Error;

const DOCKERFILE_CONTENT: &str = include_str!("../../Dockerfile");

pub fn image_needs_to_be_built(tag: &str) -> Result<bool, Error> {
    let tag_id = cmd!(
        "docker",
        "image",
        "ls",
        "--quiet",
        "--filter",
        format!("reference={}", tag)
    )
    .read()?;

    Ok(tag_id.is_empty())
}

pub fn build_image(tag: &str) -> Result<(), Error> {
    cmd!("echo", DOCKERFILE_CONTENT)
        .pipe(cmd!("docker", "build", "-t", tag, "-"))
        .run()?;

    Ok(())
}

#[allow(dead_code)]
pub fn prune_containers() -> Result<usize, Error> {
    let container_output = cmd!(
        "docker",
        "container",
        "ls",
        "--filter",
        "name=fake-ci",
        "--quiet"
    )
    .pipe(cmd!("xargs", "docker", "container", "rm", "-f"))
    .read()?;

    let container_lines = container_output
        .split('\n')
        .filter(|s| !s.is_empty())
        .count();

    Ok(container_lines)
}

#[allow(dead_code)]
pub fn prune_volumes() -> Result<usize, Error> {
    let volume_output = cmd!("docker", "volume", "ls", "--filter", "name=fake", "--quiet")
        .pipe(cmd!("xargs", "docker", "volume", "rm", "-f"))
        .read()?;

    let volume_lines = volume_output.split('\n').filter(|s| !s.is_empty()).count();

    Ok(volume_lines)
}

#[allow(dead_code)]
pub fn prune_images() -> Result<usize, Error> {
    let image_output = cmd!(
        "docker",
        "image",
        "ls",
        "--filter",
        "reference=fake-ci",
        "--quiet"
    )
    .pipe(cmd!("xargs", "docker", "image", "rm", "-f"))
    .read()?;

    let image_lines = image_output.split('\n').filter(|s| !s.is_empty()).count();

    Ok(image_lines)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    const TEST_IMAGE_TAG: &str = "fake-ci:test";

    fn clean_up_test_artifacts() {
        cmd!(
            "docker",
            "image",
            "ls",
            "--filter",
            format!("reference={}", TEST_IMAGE_TAG),
            "--quiet"
        )
        .pipe(cmd!("xargs", "docker", "image", "rm", "-f"))
        .run()
        .unwrap();
    }

    #[test]
    #[cfg_attr(not(feature = "docker_tests"), ignore)]
    fn identifies_image_tags_that_need_to_be_built() {
        assert!(image_needs_to_be_built("tag-that-does-not-exist").unwrap());
    }

    #[test]
    #[cfg_attr(not(feature = "docker_tests"), ignore)]
    fn builds_new_images_successfully() {
        clean_up_test_artifacts();

        build_image(TEST_IMAGE_TAG).unwrap();

        let new_image_id = cmd!(
            "docker",
            "image",
            "ls",
            "--quiet",
            "--filter",
            format!("reference={}", TEST_IMAGE_TAG),
        )
        .read()
        .unwrap();

        assert!(!new_image_id.is_empty());

        clean_up_test_artifacts();
    }
}
