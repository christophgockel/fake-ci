# Fake CI

Run your CI pipelines locally for quicker iterations.


## Development Notes

This tool is currently in active development.
While validating the approach I'm focussing on support for GitLab exclusively.
Eventually support for more CI/CD providers is intended like GitHub Actions, CircleCI and others.


## Build Steps

Eventually Fake CI can, and potentially should, be its own dedicated tool.
But while still validating the overall idea it consists of a few shell scripts.

There are a few manual commands necessary to get it running:

```
# this alias needs to be defined in your shell
alias fake-ci=~/<path to>/fake-ci/fake-ci.sh

# with the alias defined navigate to a project directory
# containing a .gitlab-ci.yml file
cd <project directory>

# From there you can invoke Fake CI
fake-ci
```

The `fake-ci` shell script automatically builds a Docker image it needs in case it's not available yet.


### Required Tools

- Docker or Rancher
- Git
- macOS (not tested on anything else)


## Concepts

Fake CI consists of a family of containers for different tasks that are involved throughout a CI pipeline run.

- **Checkout Containers** initialise the Git repository with the project's code and applies any pending changes to always have the latest code available.
- **Preparation Containers** take what a Checkout Container has prepared and combines it with data from Artifact and Cache Volumes if required.
- **Job Containers** run the individual CI jobs on their respective images. Content is shared from the Preparation Container.
- **Artifact Volumes** store artifacts shared between Job Containers.
- **Cache Volumes** are similar to Artifact Volumes in that CI pipelines can use caches between jobs to reduce overall runtime and required reprocessing.

The main difference between artifacts and caches are that caches are byproduct of CI jobs to support additional jobs or subsequent pipeline runs.
Whereas artifacts are explicit outputs from a job like a final JAR file for a Java project.
