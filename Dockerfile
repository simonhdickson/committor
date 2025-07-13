# Multi-stage Dockerfile for Committor
# This creates an optimized container image for the Committor CLI tool

# Build stage
FROM rust:1.75-slim as builder

# Install system dependencies needed for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy the actual source code
COPY src ./src
COPY examples ./examples
COPY tests ./tests

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 committor

# Copy the binary from builder stage
COPY --from=builder /app/target/release/committor /usr/local/bin/committor

# Make binary executable
RUN chmod +x /usr/local/bin/committor

# Switch to non-root user
USER committor

# Set working directory to user home
WORKDIR /home/committor

# Set up git config (can be overridden at runtime)
RUN git config --global user.name "Committor Bot" && \
    git config --global user.email "committor@example.com"

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD committor --help || exit 1

# Default command
ENTRYPOINT ["committor"]
CMD ["--help"]

# Metadata
LABEL maintainer="Committor Team"
LABEL description="AI-powered conventional commit message generator"
LABEL version="0.1.0"
LABEL org.opencontainers.image.source="https://github.com/yourusername/committor"
LABEL org.opencontainers.image.documentation="https://github.com/yourusername/committor/blob/main/README.md"
LABEL org.opencontainers.image.licenses="MIT"
