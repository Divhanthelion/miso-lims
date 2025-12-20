# MISO LIMS Rust Server
# Multi-stage build for minimal production image

# Stage 1: Build
FROM rust:1.83-bookworm as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/miso-domain/Cargo.toml crates/miso-domain/
COPY crates/miso-application/Cargo.toml crates/miso-application/
COPY crates/miso-infrastructure/Cargo.toml crates/miso-infrastructure/
COPY crates/miso-api/Cargo.toml crates/miso-api/
COPY crates/miso-migration/Cargo.toml crates/miso-migration/
COPY crates/miso-frontend/Cargo.toml crates/miso-frontend/

# Create dummy source files for dependency caching
RUN mkdir -p crates/miso-domain/src && echo "pub fn dummy() {}" > crates/miso-domain/src/lib.rs
RUN mkdir -p crates/miso-application/src && echo "pub fn dummy() {}" > crates/miso-application/src/lib.rs
RUN mkdir -p crates/miso-infrastructure/src && echo "pub fn dummy() {}" > crates/miso-infrastructure/src/lib.rs
RUN mkdir -p crates/miso-api/src && echo "fn main() {}" > crates/miso-api/src/main.rs && echo "pub fn dummy() {}" > crates/miso-api/src/lib.rs
RUN mkdir -p crates/miso-migration/src && echo "fn main() {}" > crates/miso-migration/src/main.rs && echo "pub fn dummy() {}" > crates/miso-migration/src/lib.rs
RUN mkdir -p crates/miso-frontend/src && echo "pub fn dummy() {}" > crates/miso-frontend/src/lib.rs

# Build dependencies (this layer is cached)
RUN cargo build --release --bin miso-server 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/

# Touch source files to invalidate cache
RUN touch crates/miso-domain/src/lib.rs
RUN touch crates/miso-application/src/lib.rs
RUN touch crates/miso-infrastructure/src/lib.rs
RUN touch crates/miso-api/src/lib.rs
RUN touch crates/miso-api/src/main.rs
RUN touch crates/miso-migration/src/lib.rs
RUN touch crates/miso-migration/src/main.rs
RUN touch crates/miso-frontend/src/lib.rs

# Build the actual application
RUN cargo build --release --bin miso-server
RUN cargo build --release --bin miso-migrate

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/miso-server /app/
COPY --from=builder /app/target/release/miso-migrate /app/

# Create non-root user
RUN useradd -r -s /bin/false miso
USER miso

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the server
CMD ["./miso-server"]

