FROM rustlang/rust:nightly-buster AS builder
# ref. https://github.com/rust-lang/docker-rust-nightly

WORKDIR /app

COPY ./Cargo.toml ./Cargo.lock ./

# minimum compilable main.rs
RUN mkdir src
RUN echo "fn main(){}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/preview_forwarder*

COPY ./src ./src
RUN cargo build --release

FROM debian:10.6-slim

COPY --from=builder /app/target/release/preview-forwarder /app/target/release/preview-forwarder
CMD ROCKET_PORT=80 /app/target/release/preview-forwarder
