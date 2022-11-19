# Fake CI

Run your CI pipelines locally for quicker iterations and faster feedback.


## Development Notes

This tool is currently in active development.
While validating the approach I'm focussing on support for GitLab only.
Eventually support for more CI/CD providers is intended like GitHub Actions, CircleCI and others.


### Required Tools

- Docker or Rancher
- Git
- yq
- macOS (not tested on anything else)


### Development and Test Setup

During development there are a lot of hard coded assumptions in the individual scripts.
This is intentional and helps validating ideas while not having to implement a lot of logic upfront that might not be necessary after learning more about it.

However, ultimately any hard coded parts need to be replaced with logic depending on a real `.gitlab-ci.yml` file.
To aid development and testing you can execute the script `initialise-example-repo.sh` that's in this repository.
The repository that is created from this acts as the reference currently being developed towards.

```
mkdir -p ~/path/to/directory/for/example/repo
./initialise-example-repo.sh ~/path/to/directory/for/example/repo

cd ~/path/to/directory/for/example/repo
fake-ci
```

Make sure to create the example _outside_ of the directory where you cloned this repository to.
This is because `fake-ci` expects the `.gitlab-ci.yml` configuration to exist in the root of the repository.
Supporting custom paths for this is a future use case that is currently not a priority.


#### Integration Tests

There are some tests that verify the overall functionality of the `fake-ci` binary.
These tests are located in the top-level `tests` directory.


##### Docker

Some tests interact directly with Docker and are separated by a feature flag which is disabled by default when running `cargo test`.

Running the Docker specific tests can be done via:

```
cargo test docker --features docker_tests
```


## Build Steps

Eventually Fake CI can, and potentially should, be its own dedicated tool.
But while still validating the overall idea it consists of a few shell scripts and a binary written in Rust.

There are a few manual commands necessary to get it running:

```
# compile the binary
cargo build

# this alias needs to be defined in your shell
alias fake-ci=~/<path to>/fake-ci/fake-ci.sh

# with the alias defined navigate to a project directory
# containing a .gitlab-ci.yml file
cd <project directory>

# From there you can invoke Fake CI
fake-ci
```

The `fake-ci` shell script automatically builds a Docker image it needs in case it's not available yet.

Compiling the binary with `cargo build` is necessary to create the binary file the remaining shell scripts expect to exist.


## Concepts

Fake CI consists of a family of containers for different tasks that are involved throughout a CI pipeline run.

- **Checkout Containers** initialise the Git repository with the project's code and applies any pending changes to always have the latest code available.
- **Preparation Containers** take what a Checkout Container has prepared and combines it with data from Artifact and Cache Volumes if required.
- **Job Containers** run the individual CI jobs on their respective images. Content is shared from the Preparation Container.
- **Artifact Volumes** store artifacts shared between Job Containers.
- **Cache Volumes** are similar to Artifact Volumes in that CI pipelines can use caches between jobs to reduce overall runtime and required reprocessing.

The main difference between artifacts and caches are that caches are a byproduct of CI jobs to support additional jobs or subsequent pipeline runs.
Whereas artifacts are explicit outputs from a job like a final JAR file for a Java project.
