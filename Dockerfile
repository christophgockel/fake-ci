FROM alpine:latest

RUN apk add git --no-cache

COPY ./gitconfig /root/.gitconfig

ENTRYPOINT ["sh"]
