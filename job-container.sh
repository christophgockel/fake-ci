#!/bin/zsh

set -euo pipefail

job_name="$1"

commands_to_run="
  set -x;
  cd /job;
"

fake_ci_directory=$(dirname "$0")
fake_ci_binary="${fake_ci_directory}/target/debug/fake-ci"

before_script='(.["'"${job_name}"'"].before_script // [])'
script='(.["'"${job_name}"'"].script // [])'
after_script='(.["'"${job_name}"'"].after_script // [])'
all_scripts="${before_script}"' + '"${script}"' + '"${after_script}"' | .[]'

script_lines=$(yq "${all_scripts}" <("$fake_ci_binary"))

variables=$(yq '(.["'"${job_name}"'"].variables // []) | to_entries | .[] | "export " + .key +"=\"" + .value + "\";" | . style="double"' <("${fake_ci_binary}"))
commands_to_run+="${variables}"

image_value=$(yq '.["'"${job_name}"'"].image' <("${fake_ci_binary}"))
interpolated_image_name=$(docker run \
  --tty \
  --rm \
  alpine:latest \
  sh -c "${variables} echo \"${image_value}\"" | sed 's/\r//') # sed is necessary due to subshell output having the \n removed

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
  "${interpolated_image_name}"

# execute job's commands
docker exec \
  fake-ci-job \
  sh -c "${commands_to_run/$'\n'/}"


# after the job finished successfully get optional artifacts out
# further cache steps to be added later
artifact_paths=$(yq '.["'"${job_name}"'"].artifacts.paths[]' <("$fake_ci_binary"))

if [ -n "$artifact_paths" ]
then
  echo "Extracting Artifacts."

  commands_to_run="
    cd /job;
    mkdir -p \"/artifacts/${job_name}\";
  "

  while IFS= read -r line
  do
    commands_to_run+="cp -R ./${line} \"/artifacts/${job_name}/\";"
  done < <(echo "$artifact_paths")

  docker exec \
    fake-ci-job \
    sh -c "${commands_to_run/$'\n'/}"
else
  echo "No Artifacts to Extract."
fi
