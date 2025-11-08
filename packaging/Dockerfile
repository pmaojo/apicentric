# syntax=docker/dockerfile:1

# Build stage
FROM --platform=$BUILDPLATFORM rust:1-bookworm as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin apicentric-cloud

# Runtime stage
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/apicentric-cloud /usr/local/bin/apicentric-cloud
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/apicentric-cloud"]
CMD ["--port", "8080"]
