FROM rustlang/rust:nightly AS base
WORKDIR build
RUN cargo install cargo-chef

FROM base AS plan
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS build
# build dependencies, this can be cached
COPY --from=plan /build/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json
# build the app
COPY . .
RUN cargo build --release

FROM debian:stable-slim
WORKDIR app
COPY --from=build /build/target/release/battlesnake-doctor-strangle /usr/local/bin
ENV RUST_LOG debug
ENTRYPOINT ["battlesnake-doctor-strangle"]
