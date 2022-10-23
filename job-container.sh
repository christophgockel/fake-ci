#!/bin/sh

set -euo pipefail

job_name="$1"

#
# This container simulates running CI jobs with the following configuration
# depending on the argument to the script.
#
# build:
#   stage: build
#   image: alpine:latest
#   script:
#     - echo "Running build."
#     - cat readme.md
#     - echo "message" > file.txt
#   artifacts:
#     paths:
#       - file.txt
#
# test:
#   stage: test
#   image: alpine:latest
#   script:
#     - echo "Running test."
#     - cat file.txt
#   needs:
#     - job: build
#       artifacts: true
#

# further cache steps to be added later
if [ "$job_name" == "build" ]
then
  commands="
    set -x;
    cd /job;
    echo \"Running build\";
    cat readme.md;
    echo \"message\" > file.txt
  "
else
  commands="
    set -x;
    cd /job;
    echo \"Running test\";
    cat file.txt;
  "
fi


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
if [ "$job_name" == "build" ]
then
  commands="
  cd /job;
  mkdir -p /artifacts/build;
  cp file.txt /artifacts/build/;
  "

  docker exec \
    fake-ci-job \
    sh -c "${commands/$'\n'/}"
fi
