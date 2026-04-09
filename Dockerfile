# ── Build stage ──
FROM rust:1.86-slim AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release && strip target/release/sq

# ── Runtime stage ──
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    tini \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/sq /usr/local/bin/sq

# Create a non-root user for the adventure
RUN useradd -m -s /bin/bash adventurer
USER adventurer
WORKDIR /home/adventurer

# Pre-create the save directory
RUN mkdir -p /home/adventurer/.shellquest

# Install the bash hook
RUN sq hook --shell bash --install

ENTRYPOINT ["tini", "--"]
CMD ["bash"]
