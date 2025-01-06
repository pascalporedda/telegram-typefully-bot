# Builder stage
FROM rust:1.83.0-slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

# Copy the Cargo files first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# # Create a dummy main.rs to build dependencies
# RUN mkdir src && \
#   echo "fn main() { println!(\"Hello, world!\"); }" > src/main.rs && \
#   cargo build --release && \
#   rm -rf src

# Copy the actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
  ca-certificates \
  libssl3 \
  && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/telegram-typefully-bot /app/telegram-typefully-bot

# # Create necessary directories
# RUN mkdir -p /app/voice-notes && \
#   touch /app/bot.db && \
#   chmod 755 /app/voice-notes && \
#   chmod 644 /app/bot.db

CMD ["/app/telegram-typefully-bot"]
