FROM rust:1-slim-bullseye AS builder
RUN apt update -qq && apt install -y -qq --no-install-recommends \
    perl gcc make
RUN rustup set profile minimal
WORKDIR /usr/src/memosy_bot_rust
COPY . .
ENV RUSTFLAGS='-C link-arg=-s'
RUN cargo build --release

FROM lwthiker/curl-impersonate:0.6-chrome-slim-bullseye
RUN apt update -qq && apt install -y -qq --no-install-recommends \
    ca-certificates
COPY --from=builder /usr/src/memosy_bot_rust/target/release/memosy_bot_rust /usr/local/bin/memosy_bot_rust
CMD ["memosy_bot_rust"]
