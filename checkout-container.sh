#!/bin/zsh

set -euo pipefail

# to be figured out: cleaning/removing existing files and update with latest version.
# the last `git apply` gets any pending changes so that changes don't have to be committed to be part of the run.
commands_to_run="
  cd /checkout;
  git init;
  git remote add origin /project;
  git fetch origin --quiet;
  git checkout --quiet \${GIT_SHA};
  (cd /project; git add --intent-to-add .; git diff) | git apply --allow-empty --quiet;
  (cd /project; git reset --mixed)
"

docker ps --all --quiet --filter name=fake-ci-checkout | xargs docker rm --force > /dev/null

docker run \
  --tty \
  --detach \
  --volume $(pwd):/project \
  --volume /checkout \
  --name fake-ci-checkout \
  fake-ci:latest

docker exec \
  --env GIT_SHA=$(git rev-parse HEAD) \
  fake-ci-checkout \
  sh -c "${commands_to_run/$'\n'/}"
