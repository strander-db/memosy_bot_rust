FROM rust:1-slim AS builder
RUN apt update -qq && apt install -y -qq --no-install-recommends \
    musl-tools perl gcc make
RUN rustup set profile minimal && rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src/memosy_bot_rust
COPY . .
ENV RUSTFLAGS='-C link-arg=-s'
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM lwthiker/curl-impersonate:0.6-chrome-alpine
RUN apk add --no-cache ca-certificates
COPY --from=builder /usr/src/memosy_bot_rust/target/x86_64-unknown-linux-musl/release/memosy_bot_rust /usr/local/bin/memosy_bot_rust
CMD ["memosy_bot_rust"]
