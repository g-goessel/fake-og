FROM rust:1.40 as builder
RUN USER=root cargo new --bin fake-og
WORKDIR /fake-og
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# let's cache the dependencies
RUN apt-get update && apt-get install -y libclang-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release
RUN rm src/*.rs

# now we add our code and compile it
COPY ./src ./src
COPY ./templates ./templates
RUN rm ./target/release/deps/fake_og*
RUN cargo build --release

# at the end we build the final image
FROM ubuntu:18.04
RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /fake-og
COPY --from=builder /fake-og/target/release/fake-og /fake-og/fake-og
COPY ./migrations /fake-og/migrations
COPY ./static /fake-og/static
COPY ./templates /fake-og/templates
CMD ["./fake-og"]