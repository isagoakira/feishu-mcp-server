FROM rust:1.75-slim

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock* ./
COPY src/ ./src/

# Build dependencies and application
RUN cargo build --release

# Expose the default port
EXPOSE 3000

# Run the server
CMD ["target/release/feishu-mcp-server", "--port", "3000"]