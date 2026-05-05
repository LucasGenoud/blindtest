# Build stage
FROM rust:1.88-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy actual source
COPY src/ src/
COPY migrations/ migrations/

# Build the real binary
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates wget ffmpeg python3 && rm -rf /var/lib/apt/lists/*
RUN wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp && chmod a+rx /usr/local/bin/yt-dlp

COPY --from=builder /app/target/release/blindtest-server .
COPY migrations/ migrations/

# Secret keys will be mounted via docker-compose
RUN mkdir -p secret data

ENV PORT=80
EXPOSE 80

CMD ["/app/blindtest-server"]
