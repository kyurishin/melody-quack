# The relay server only. Deploy anywhere that runs containers, then put its
# ws:// (or wss:// behind TLS) address in the game's SERVER field.
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p mq-server

FROM debian:bookworm-slim
COPY --from=build /src/target/release/mq-server /usr/local/bin/mq-server
ENV PORT=7777
EXPOSE 7777
CMD ["mq-server"]
