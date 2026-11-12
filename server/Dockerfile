FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=build /src/target/release/melody-quack-server /usr/local/bin/melody-quack-server
ENV PORT=7777
EXPOSE 7777
CMD ["melody-quack-server"]
