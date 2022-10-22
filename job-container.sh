#!/bin/sh

set -euxo pipefail

#
# This job simulates running a CI job with the following configuration
#
# build-job:
#   stage: build
#   image: alpine:latest
#   script:
#     - echo "Running build-job."
#     - cat readme.md
#

# further artifacts and cache steps to be added later
commands="
cd /job;
echo \"Running build-job\";
cat readme.md;
"

docker run \
  --tty \
  --rm \
  --volumes-from fake-ci-preparation \
  --name fake-ci-job \
  alpine:latest \
  sh -c "${commands/$'\n'/}"
