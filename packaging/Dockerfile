# syntax=docker/dockerfile:1

# Build stage
FROM --platform=$BUILDPLATFORM rust:1-bookworm as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/mockforge /usr/local/bin/mockforge
ENTRYPOINT ["/usr/local/bin/mockforge"]
