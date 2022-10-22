#!/bin/sh

set -euo pipefail

# further artifacts and cache steps to be added later
commands="
cp -Rp /checkout/. /job;
sh"

docker ps -aq --filter name=fake-ci-preparation | xargs docker rm -f > /dev/null

# run this on the base image because it cannot be guaranteed that a job's image has `cp` available
docker run \
  --interactive \
  --tty \
  --detach \
  --volumes-from fake-ci-checkout \
  --volume /job \
  --name fake-ci-preparation \
  fake-ci:latest \
  "${commands/$'\n'/}"
