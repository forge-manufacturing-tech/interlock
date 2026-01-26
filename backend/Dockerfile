FROM rust:1-slim-bookworm AS chef
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/app/recipe.json recipe.json
# Build dependencies - this is the caching layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /usr/app
# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/* && \
    mkdir -p /root/.postgresql && cp /etc/ssl/certs/ca-certificates.crt /root/.postgresql/root.crt
COPY --from=builder /usr/src/app/target/release/backend-cli /usr/app/backend-cli
COPY config /usr/app/config
ENV LOCO_ENV=production
EXPOSE 5150
CMD ["./backend-cli", "start"]
