FROM alpine:latest

RUN apk add git --no-cache

COPY ./gitconfig /root/.gitconfig
COPY ./entrypoint.sh /

CMD ["sh"]
ENTRYPOINT ["/entrypoint.sh"]
