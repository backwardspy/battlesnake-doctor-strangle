FROM rust:latest AS build
WORKDIR build
COPY . .
RUN cargo build --release

FROM ubuntu:latest
WORKDIR app
COPY --from=build /build/target/release/battlesnake-doctor-strangle ./battlesnake-doctor-strangle
ENV RUST_LOG debug
CMD ["./battlesnake-doctor-strangle"]
