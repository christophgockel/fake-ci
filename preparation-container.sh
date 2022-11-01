#!/bin/zsh

set -euo pipefail

job_name="$1"

fake_ci_directory=$(dirname "$0")
fake_ci_binary="${fake_ci_directory}/target/debug/fake-ci"

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
# cache steps to be added later
job_names_with_artifacts=$(yq '.["'"${job_name}"'"].needs[] | select(.artifacts == true) | .job' <("$fake_ci_binary"))

if [ -n "$job_names_with_artifacts" ]
then
  echo "Preparing artifacts."

  commands_to_run=""

  while IFS= read -r job_name_of_artifact
  do
    artifact_paths=$(yq '.["'"${job_name_of_artifact}"'"].artifacts.paths[]' <("$fake_ci_binary"))

    if [ -n "$artifact_paths" ]
    then
      while IFS= read -r artifact_path
      do
        commands_to_run+="cp -Rp \"/artifacts/${job_name_of_artifact}/${artifact_path}\" /job;"
      done < <(echo "$artifact_paths")
    fi

  done < <(echo "$job_names_with_artifacts")

  docker exec \
    fake-ci-preparation \
    sh -c "${commands_to_run/$'\n'/}"
else
  echo "Skipping artifacts."
fi
