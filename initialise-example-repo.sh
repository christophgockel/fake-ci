#!/bin/zsh

set -euo pipefail

target_directory=${1:-}

if [ -z "$target_directory" ]
then
  echo "Missing target directory."
  echo
  echo "Usage:"
  echo "    ${0} ~/path/to/directory"
  exit 1
fi


if [ ! -d "$target_directory" ]
then
  echo "Target directory does not exist."
  echo "Please create it first."
  echo
  echo "Then call this script again:"
  echo "    ${0} ${target_directory}"
  exit 1
fi

cd "$target_directory"
git init

cat << EOF > readme.md
# Fake CI Test Repository
EOF

cat << EOF > .gitlab-ci.yml
stages:
  - build
  - test

build:
  stage: build
  image: alpine:latest
  script:
    - echo "Running build-job."
    - cat readme.md
    - echo "message from build step" > file.txt
  artifacts:
    paths:
      - file.txt

test:
  stage: test
  image: alpine:latest
  script:
    - echo "Running test-job."
    - cat file.txt
  needs:
    - job: build
      artifacts: true
EOF

git add .
git commit --message "Initialise Fake CI Example Repository"
