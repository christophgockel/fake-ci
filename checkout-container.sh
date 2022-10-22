#!/bin/sh

set -euo pipefail

# to be figured out: cleaning/removing existing files and update with latest version.
# the last `git apply` gets any pending changes so that changes don't have to be committed to be part of the run.
commands="
cd /checkout;
git init --quiet;
git remote add origin /project;
git fetch origin --quiet;
git checkout --quiet \${GIT_SHA};
(cd /project; git diff) | git apply --quiet;
sh"

docker ps --all --quiet --filter name=fake-ci-checkout | xargs docker rm --force > /dev/null

docker run \
  --interactive \
  --tty \
  --detach \
  --volume $(pwd):/project \
  --volume /checkout \
  --env GIT_SHA=$(git rev-parse HEAD) \
  --name fake-ci-checkout \
  fake-ci:latest \
  "${commands/$'\n'/}"
