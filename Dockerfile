FROM --platform=$BUILDPLATFORM rust:1.83.0-slim-bookworm AS builder

WORKDIR /app


# Install cross-compilation dependencies
RUN dpkg --add-architecture amd64 && \
  apt-get update && \
  apt-get install -y --no-install-recommends \
  g++-x86-64-linux-gnu \
  libc6-dev-amd64-cross

# Set cross-compilation environment variables
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
  CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc \
  CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++

# Add target 
RUN rustup target add x86_64-unknown-linux-gnu 

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-gnu

FROM --platform=$BUILDPLATFORM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/telegram-typefully-bot .

ENV DATABASE_URL=sqlite:bot.db
ENV RUST_LOG=telegram_typefully_bot=info,hyper=warn,sqlx::query=info,teloxide=debug

CMD ["./telegram-typefully-bot"]
