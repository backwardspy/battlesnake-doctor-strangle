# compile app
FROM rustlang/rust:nightly AS build

WORKDIR build

COPY . .

RUN cargo build --release --bin snake

# copy into runtime env
FROM debian:stable-slim

WORKDIR app

COPY --from=build /build/target/release/snake /usr/local/bin

ENV RUST_LOG debug

ENTRYPOINT ["snake"]
