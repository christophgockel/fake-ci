#!/bin/sh

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

job_name=${1:-}
available_jobs_list=$(yq 'keys' .gitlab-ci.yml | grep --invert-match -E "stages")
available_jobs_csv=",$(echo "$available_jobs_list" | yq 'to_csv'),"

if [ -z "$job_name" ]
then
  echo "Missing job parameter."
  echo
  echo "Available jobs:"
  echo "$available_jobs_list"
  echo
  echo "Usage:"
  echo "    fake-ci <job-name>"
  exit 1
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
fake_ci_directory=$(dirname "$0")

if [ -z "$fake_ci_image_id" ]
then
  echo "Fake CI image not found. Building now."
  docker build -t fake-ci:latest "$fake_ci_directory"
fi

echo "Checking out Code"
"$fake_ci_directory"/checkout-container.sh

echo "Preparing Code"
"$fake_ci_directory"/preparation-container.sh

echo "Running Job"
"$fake_ci_directory"/job-container.sh "$job_name"
