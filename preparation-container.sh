#!/bin/zsh

set -euo pipefail

job_name="$1"

# further artifacts and cache steps to be added later
commands_to_run="
  cp -Rp /checkout/. /job;
"

docker ps -aq --filter name=fake-ci-preparation | xargs docker rm -f > /dev/null

# run this on the base image because it cannot be guaranteed that a job's image has `cp` available
docker run \
  --tty \
  --detach \
  --volumes-from fake-ci-checkout \
  --volume fake-ci-artifacts:/artifacts \
  --volume /job \
  --name fake-ci-preparation \
  fake-ci:latest

docker exec \
  fake-ci-preparation \
  sh -c "${commands_to_run/$'\n'/}"

# after copying the code get the artifacts in place
if [ "$job_name" = "test" ]
then
  echo "Preparing artifacts."
  commands_to_run="
    cp -Rp /artifacts/build/file.txt /job;
  "

  docker exec \
    fake-ci-preparation \
    sh -c "${commands_to_run/$'\n'/}"
else
  echo "Skipping artifacts."
fi
