# compile app
FROM rustlang/rust:nightly AS build

WORKDIR build

COPY . .

RUN cargo build --release

# copy into runtime env
FROM debian:stable-slim

WORKDIR app

COPY --from=build /build/target/release/battlesnake-doctor-strangle /usr/local/bin

ENV RUST_LOG debug

ENTRYPOINT ["battlesnake-doctor-strangle"]
