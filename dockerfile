# Use a minimal base image
FROM debian:bullseye-slim

# Install necessary build tools and libraries
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    llvm \
    libsnappy-dev \
    liblz4-dev \
    libzstd-dev \
    curl \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and Cargo
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH=/root/.cargo/bin:$PATH

# Set the working directory inside the container
WORKDIR /app

# Copy the source code to the container
COPY . .

# Ensure the RocksDB directory does not already exist
RUN rm -rf /app/diskdb

# Build the project
RUN cargo build --release --bin diskdb

# Copy the compiled binary from the release folder to the container
RUN cp target/release/diskdb /app/

# Ensure the binary has execute permissions
RUN chmod +x /app/diskdb

# Expose port 6380 for testing
EXPOSE 6380

# Set the entrypoint to run the binary
CMD ["/app/diskdb"]