# syntax=docker/dockerfile:1

# Build stage
FROM --platform=$BUILDPLATFORM rust:1-bookworm as builder
WORKDIR /app
COPY . .
# Build with all features for cloud deployment
RUN cargo build --release --bin apicentric --features "simulator,contract-testing,database,file-watch,websockets"

# Runtime stage
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/apicentric /usr/local/bin/apicentric-cloud
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/apicentric-cloud"]
CMD ["cloud", "server", "--port", "8080"]
