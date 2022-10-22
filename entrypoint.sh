#!/bin/sh

set -euo pipefail

#
# Entrypoint that allows execution of single commands or a shell script passed as one argument.
#
# Passing single commands can include arguments to that command:
#
#     docker run -it --rm fake-ci:latest git --version
#
# Alternatively a multi command string can be passed in form of a string as well:
#
#     docker run -it --rm fake-ci:latest 'echo contents; ls -la'
#

if ! command -v "$@" &> /dev/null
then
  # passed argument is not an existing command so we'll run it as a shell script
  exec sh -c "$@"
else
  # passed argument is an existing command, so we'll execute it
  exec "$@"
fi
