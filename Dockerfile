FROM rust:1-slim-bullseye AS builder
RUN apt update -qq && apt install -y -qq --no-install-recommends \
    perl gcc make
RUN rustup set profile minimal
WORKDIR /usr/src/memosy_bot_rust
COPY . .
ENV RUSTFLAGS='-C link-arg=-s'
RUN cargo build --release

FROM debian:bullseye-slim AS base
RUN apt update -qq && apt install -y -qq --no-install-recommends \
    ca-certificates

# Add curl-impersonate layer just for yt-dlp
FROM lwthiker/curl-impersonate:0.6-chrome-slim-bullseye as curl-impersonate

# Final image
FROM base
COPY --from=builder /usr/src/memosy_bot_rust/target/release/memosy_bot_rust /usr/local/bin/memosy_bot_rust
# Copy only the necessary curl-impersonate files
COPY --from=curl-impersonate /usr/local/lib/libcurl-impersonate* /usr/local/lib/
COPY --from=curl-impersonate /usr/local/bin/curl-impersonate* /usr/local/bin/
# COPY --from=curl-impersonate /usr/local/lib/chrome-libs/ /usr/local/lib/chrome-libs/
# Set up the library path for curl-impersonate
ENV LD_LIBRARY_PATH=/usr/local/lib:/usr/local/lib/chrome-libs

# Create a wrapper script for yt-dlp that uses curl-impersonate
RUN echo '#!/bin/sh\n\
    export CURL="curl-impersonate-chrome"\n\
    exec "$@"' > /usr/local/bin/with-chrome-impersonate && \
    chmod +x /usr/local/bin/with-chrome-impersonate

CMD ["memosy_bot_rust"]
