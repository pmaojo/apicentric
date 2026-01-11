
# Stage 1: Build the apicentric binary
FROM rust:1.78 as builder

# Install apicentric from crates.io
# This makes the Dockerfile portable and not dependent on local source code.
# You can pin to a specific version using --version <VERSION>
RUN cargo install apicentric --no-default-features --features simulator

# Stage 2: Create the final minimal image
FROM debian:buster-slim

# Copy the apicentric binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/apicentric /usr/local/bin/apicentric

# Create a directory for the service definitions
WORKDIR /app
COPY --chown=root:root services/ ./services/

# Expose the ports from the service definitions
EXPOSE 9005

# Run the apicentric simulator, pointing to the services directory
ENTRYPOINT ["apicentric", "simulator", "start", "--services-dir", "./services"]
