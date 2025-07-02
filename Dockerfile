# Multi-stage build for Polymarket MCP Server
FROM rust:1.87-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml ./
COPY Cargo.lock* ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release && rm -rf src target/release/deps/polymarket_mcp*

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd --create-home --shell /bin/bash mcp

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/polymarket-mcp /usr/local/bin/polymarket-mcp

# Copy any additional files if needed
# COPY config/ ./config/

# Change ownership to mcp user
RUN chown -R mcp:mcp /app

# Switch to non-root user
USER mcp

# Health check - simple process check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep polymarket-mcp || exit 1

# Expose port (if needed for HTTP health checks)
EXPOSE 8080

# Set default environment variables
ENV RUST_LOG=info
ENV POLYMARKET_CACHE_TTL=300
ENV POLYMARKET_MAX_CONCURRENT_REQUESTS=10

# Run the MCP server
CMD ["polymarket-mcp"]