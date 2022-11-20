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
  "${fake_ci_binary}" run $job_name
}

subcommand_prune() {
  "${fake_ci_binary}" prune
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
