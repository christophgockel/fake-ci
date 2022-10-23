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
#     - echo "message" > file.txt
#   artifacts:
#     paths:
#       - file.txt
#
#

# further cache steps to be added later
commands="
cd /job;
echo \"Running build-job\";
cat readme.md;
echo \"message\" > file.txt
"

docker ps -aq --filter name=fake-ci-job | xargs docker rm -f > /dev/null

docker run \
  --tty \
  --detach \
  --volumes-from fake-ci-preparation \
  --name fake-ci-job \
  alpine:latest

# execute job's commands
docker exec \
  fake-ci-job \
  sh -c "${commands/$'\n'/}"

# after the job finished successfully get the artifacts out
commands="
cd /job;
mkdir -p /artifacts/build-job;
cp file.txt /artifacts/build-job/;
"

docker exec \
  fake-ci-job \
  sh -c "${commands/$'\n'/}"
