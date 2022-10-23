#!/bin/zsh

set -euo pipefail

job_name="$1"

commands_to_run="
  set -x;
  cd /job;
"

script_lines=$(yq ".${job_name}.script[]" .gitlab-ci.yml)

while IFS= read -r line
do
  commands_to_run+="${line};"
done < <(echo "$script_lines")

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
  sh -c "${commands_to_run/$'\n'/}"


# after the job finished successfully get optional artifacts out
# further cache steps to be added later
artifact_paths=$(yq ".${job_name}.artifacts.paths[]" .gitlab-ci.yml)

if [ -n "$artifact_paths" ]
then
  echo "Extracting Artifacts."

  commands_to_run="
    cd /job;
    mkdir -p /artifacts/${job_name};
  "

  while IFS= read -r line
  do
    commands_to_run+="cp -R ./${line} /artifacts/${job_name};"
  done < <(echo "$artifact_paths")

  docker exec \
    fake-ci-job \
    sh -c "${commands_to_run/$'\n'/}"
else
  echo "No Artifacts to Extract."
fi
