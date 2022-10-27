#!/bin/zsh

set -euo pipefail

if [ ! -d ".git" ]
then
  echo "Not in a git repository."
  exit 1
fi

if [ ! -f ".gitlab-ci.yml" ]
then
  echo "No .gitlab-ci.yml found."
  exit 1
fi

script_name=$(basename "$0")
script_name="${script_name:0:-3}" # removes `.sh` from filename
fake_ci_directory=$(dirname "$0")
fake_ci_binary="${fake_ci_directory}/target/debug/fake-ci"

subcommand_help() {
  echo "Usage:"
  echo "    ${script_name} <subcommand>"
  echo
  echo "Subcommands:"
  echo "    help   Show this usage help."
  echo "    run    Run a CI job."
  echo "    prune  Remove all Docker artifacts."
  echo
  echo "For help with each subcommand run:"
  echo "${script_name} <subcommand> [-h|--help]"
  exit 0
}

subcommand_run() {
  job_name=${1:-}
  available_jobs_list=$(yq 'keys' <("$fake_ci_binary") | grep --invert-match -E "stages")
  available_jobs_csv=",$(echo "$available_jobs_list" | yq 'to_csv'),"

  if [ -z "$job_name" ] || [ "$job_name" = "-h" ] || [ "$job_name" = "--help" ]
  then
    echo "Available jobs:"
    echo "$available_jobs_list"
    echo
    echo "Usage:"
    echo "    ${script_name} run <job-name>"
    exit 0
  fi

  if ! echo "$available_jobs_csv" | grep -e ",${job_name}," 1> /dev/null
  then
    echo "Job '${job_name}' not found."
    echo
    echo "Available jobs:"
    echo "$available_jobs_list"
    exit 1
  fi

  fake_ci_image_id=$(docker image ls --filter reference=fake-ci:latest --quiet)

  if [ -z "$fake_ci_image_id" ]
  then
    echo "Fake CI image not found. Building now."
    docker build -t fake-ci:latest "$fake_ci_directory"
  fi

  echo "Checking out Code"
  "$fake_ci_directory"/checkout-container.sh

  echo "Preparing Code"
  "$fake_ci_directory"/preparation-container.sh "$job_name"

  echo "Running Job"
  "$fake_ci_directory"/job-container.sh "$job_name"
}

subcommand_prune() {
  docker container ls --filter name=fake-ci --quiet | xargs docker container rm -f
  docker volume ls --filter name=fake --quiet | xargs docker volume rm -f
  docker image ls --filter reference=fake-ci:latest --quiet | xargs docker image rm -f
}

subcommand=${1:-}

if [ -z "$subcommand" ] || [ "$subcommand" = "-h" ] || [ "$subcommand" = "--help" ]
then
  subcommand_help
else
  shift

  if type "subcommand_${subcommand}" 2>/dev/null | grep -q 'function'
  then
    "subcommand_${subcommand}" "$@"
  else
     echo "Error: '${subcommand}' is not a known subcommand."
     echo
     subcommand_help
     exit 1
  fi
fi
