use duct::cmd;

const DOCKERFILE_CONTENT: &str = include_str!("../../Dockerfile");

pub fn build_image(tag: &str) -> Result<(), std::io::Error> {
    cmd!("echo", DOCKERFILE_CONTENT)
        .pipe(cmd!("docker", "build", "-t", tag, "-"))
        .run()?;

    Ok(())
}

pub fn image_needs_to_be_built(tag: &str) -> Result<bool, std::io::Error> {
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
