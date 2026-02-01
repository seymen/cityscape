FROM rust:1.93-slim-bookworm as build

RUN apt update && apt install -y pkg-config libssl-dev

# create a new empty shell project
RUN cargo new --bin cityscape
WORKDIR /cityscape

# copy over your manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/cityscape*
RUN cargo build --release

# our final base
FROM gcr.io/distroless/cc-debian12 AS runner

# copy the build artifact from the build stage
COPY --from=build /cityscape/target/release/cityscape .

# set the startup command to run your binary
CMD ["./cityscape"]
