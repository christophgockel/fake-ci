# Fake CI

Run your CI pipelines locally for quicker iterations.


## Development Notes

This tool is currently in active development.
While validating the approach I'm focussing on support for GitLab exclusively.
Eventually support for more CI/CD providers is intended like GitHub Actions, CircleCI and others.


## Build Steps

Without having a tool that combines and orchestrates the overall process of running a pipeline locally, this chapter lists the individual steps and commands to run a CI job manually.

Eventually this could be taken care of by a dedicated tool.
But while still validating the overall idea there will be a few manual commands and individual shell scripts that will show how all individual pieces can fit together.

```
# build the core image
docker build -t fake-ci:latest .

alias checkout-container=~/development/fake-ci/checkout-container.sh
alias preparation-container=~/development/fake-ci/preparation-container.sh

# building the checkout container in a project repository
cd <project directory>
checkout-container

# with the checkout container available the preparation container can be created
preparation-container
```


## Concepts

Fake CI consists of a family of containers for different tasks that are involved throughout a CI pipeline run.

- **Checkout Containers** initialise the Git repository with the project's code and applies any pending changes to always have the latest code available.
- **Preparation Containers** take what a Checkout Container has prepared and combines it with data from Artifact and Cache Volumes if required.
- **Job Containers** run the individual CI jobs on their respective images. Content is shared from the Preparation Container.
- **Artifact Volumes** store artifacts shared between Job Containers.
- **Cache Volumes** are similar to Artifact Volumes in that CI pipelines can use caches between jobs to reduce overall runtime and required reprocessing.

The main difference between artifacts and caches are that caches are byproduct of CI jobs to support additional jobs or subsequent pipeline runs.
Whereas artifacts are explicit outputs from a job like a final JAR file for a Java project.
