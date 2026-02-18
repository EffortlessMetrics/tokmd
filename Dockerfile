# syntax=docker/dockerfile:1

# =============================================================================
# Stage 1: Build
# =============================================================================
FROM rust:1.92-alpine AS builder

# Install build dependencies for musl-based static linking
RUN apk add --no-cache musl-dev

WORKDIR /build

# Copy workspace manifest and all crate manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY fuzz/ fuzz/
COPY xtask/ xtask/

# Build the release binary
RUN cargo build --release --bin tokmd --locked

# =============================================================================
# Stage 2: Runtime (minimal image)
# =============================================================================
FROM alpine:3.21 AS runtime

# Install ca-certificates for HTTPS support and git for repository analysis
RUN apk add --no-cache ca-certificates git

# Create non-root user for security
RUN adduser -D -u 1000 tokmd

# Copy the binary from the builder stage
COPY --from=builder /build/target/release/tokmd /usr/local/bin/tokmd

# Verify binary works
RUN tokmd --version

# Switch to non-root user
USER tokmd

# Set working directory for mounted repositories
WORKDIR /repo

# OCI Image Labels
# https://github.com/opencontainers/image-spec/blob/main/annotations.md
LABEL org.opencontainers.image.title="tokmd" \
      org.opencontainers.image.description="AI-native code inventory receipts and analytics for LLM workflows" \
      org.opencontainers.image.url="https://github.com/EffortlessMetrics/tokmd" \
      org.opencontainers.image.source="https://github.com/EffortlessMetrics/tokmd" \
      org.opencontainers.image.documentation="https://github.com/EffortlessMetrics/tokmd#readme" \
      org.opencontainers.image.vendor="EffortlessMetrics" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0" \
      org.opencontainers.image.base.name="alpine:3.21"

# Set tokmd as the entrypoint
ENTRYPOINT ["tokmd"]

# Default command shows help
CMD ["--help"]
