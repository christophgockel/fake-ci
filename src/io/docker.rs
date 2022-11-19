use duct::cmd;

const DOCKERFILE_CONTENT: &str = include_str!("../../Dockerfile");

pub fn build_image(tag: &str) -> Result<(), std::io::Error> {
    cmd!("echo", DOCKERFILE_CONTENT)
        .pipe(cmd!("docker", "build", "-t", tag, "-"))
        .run()?;

    Ok(())
}
