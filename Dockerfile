# Multi-stage Dockerfile for StepheyBot Music
# Optimized for build speed and small final image size

# Build stage
FROM rust:latest as builder

# Install build dependencies including Node.js for frontend
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    libasound2-dev \
    libavformat-dev \
    libavcodec-dev \
    libavutil-dev \
    libswresample-dev \
    libclang-dev \
    cmake \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 18.x
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

# Create app user
RUN useradd -m -u 1001 stepheybot

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml ./
COPY migrations/ ./migrations/

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached if Cargo.toml doesn't change)
RUN cargo build --release && rm -rf src target/release/deps/stepheybot_music*

# Copy source code
COPY src/ ./src/

# Build the application
RUN touch src/main.rs && cargo build --release

# Strip the binary to reduce size
RUN strip target/release/stepheybot-music

# Build frontend
COPY frontend/ ./frontend/
WORKDIR /app/frontend
RUN npm install --legacy-peer-deps
RUN npm run build

# Return to app directory
WORKDIR /app

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    libasound2 \
    libavformat59 \
    libavcodec59 \
    libavutil57 \
    libswresample4 \
    curl \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create app user and directories
RUN useradd -m -u 1001 stepheybot && \
    mkdir -p /app /data /cache /music /downloads /logs && \
    chown -R stepheybot:stepheybot /app /data /cache /music /downloads /logs

# Set working directory
WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder /app/target/release/stepheybot-music /app/stepheybot-music
COPY --from=builder /app/migrations /app/migrations

# Copy the built frontend files
COPY --from=builder /app/frontend/build /app/frontend

# Copy configuration files if they exist (optional)
# COPY config/ ./config/

# Make binary executable
RUN chmod +x /app/stepheybot-music

# Switch to app user
USER stepheybot

# Expose the default port
EXPOSE 8083

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV STEPHEYBOT__SERVER__PORT=8083
ENV STEPHEYBOT__SERVER__ADDRESS=0.0.0.0
ENV STEPHEYBOT__DATABASE__URL=sqlite:/data/stepheybot-music.db
ENV STEPHEYBOT__PATHS__CACHE_PATH=/cache
ENV STEPHEYBOT__PATHS__MUSIC_PATH=/music
ENV STEPHEYBOT__PATHS__DOWNLOAD_PATH=/downloads
ENV STEPHEYBOT__LOGGING__LEVEL=info

# Create volumes for persistent data
VOLUME ["/data", "/cache", "/music", "/downloads", "/logs"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8083/health || exit 1

# Run the application
CMD ["/app/stepheybot-music"]

# Metadata
LABEL maintainer="Stephey <stephey@stepheybot.dev>"
LABEL description="StepheyBot Music - Private Spotify-like music streaming service with AI recommendations"
LABEL version="1.0.0"
LABEL org.opencontainers.image.source="https://github.com/stephey/stepheybot-music"
LABEL org.opencontainers.image.description="Advanced music recommendation and streaming service built with Rust"
LABEL org.opencontainers.image.licenses="MIT"
