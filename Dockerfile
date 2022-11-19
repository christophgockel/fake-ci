FROM alpine:latest

RUN apk add git --no-cache

RUN git config --global init.defaultBranch none && \
  git config --global apply.whitespace nowarn && \
  git config --global safe.directory /project

ENTRYPOINT ["sh"]
