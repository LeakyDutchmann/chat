# ============================
# 1. Build stage
# ============================
FROM rustlang/rust:nightly AS builder

WORKDIR /app

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release

# ============================
# 2. Runtime stage
# ============================
FROM debian:bookworm-slim

# Install only what your Rust binary needs
RUN apt-get update && apt-get install -y libssl-dev && apt-get clean

# Create app directory
WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/main /usr/local/bin/chat

# Copy static assets
COPY --from=builder /app/static /app/static

# Expose port
EXPOSE 8080

# Set APP_ROOT so your Rust code knows where static files live
ENV APP_ROOT=/app

CMD ["chat"]