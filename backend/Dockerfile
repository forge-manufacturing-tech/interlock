FROM rust:1-slim-bookworm as builder

WORKDIR /usr/src/app

COPY . .

# Build the application
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /usr/src/app/target/release/backend-cli /usr/app/backend-cli

# Copy configuration
COPY config /usr/app/config

# Set environment to production
ENV LOCO_ENV=production

# Expose the application port
EXPOSE 5150

# Run the application
CMD ["./backend-cli", "start"]
