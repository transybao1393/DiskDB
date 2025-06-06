# DiskDB Makefile for multi-platform builds

# Version
VERSION := 0.1.0

# Detect OS and architecture
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Rust build settings
CARGO := cargo
RUSTFLAGS := -C target-cpu=native

# Define targets
TARGET_DIR = target/release
BINARY_NAME = diskdb

# Cross-compilation targets
LINUX_X86_64 = x86_64-unknown-linux-gnu
LINUX_AARCH64 = aarch64-unknown-linux-gnu
MACOS_X86_64 = x86_64-apple-darwin
MACOS_AARCH64 = aarch64-apple-darwin

# Build directory
BUILD_DIR := builds

.PHONY: all build release test clean install package-all package-python

# Default target
all: build

# Development build
build:
	cargo build --release

# Debug build
debug:
	cargo build

# Debug run
debug-run:
	cargo run

# Create platform-specific releases
release-macos-intel:
	@echo "Building for macOS Intel..."
	@mkdir -p $(BUILD_DIR)
	rustup target add $(MACOS_X86_64)
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --release --target $(MACOS_X86_64)
	@mkdir -p $(BUILD_DIR)/diskdb-$(VERSION)-macos-x86_64
	@cp target/$(MACOS_X86_64)/release/diskdb $(BUILD_DIR)/diskdb-$(VERSION)-macos-x86_64/
	@cp README.md $(BUILD_DIR)/diskdb-$(VERSION)-macos-x86_64/
	@tar -czf $(BUILD_DIR)/diskdb-$(VERSION)-macos-x86_64.tar.gz -C $(BUILD_DIR) diskdb-$(VERSION)-macos-x86_64

release-macos-arm:
	@echo "Building for macOS ARM (Apple Silicon)..."
	@mkdir -p $(BUILD_DIR)
	rustup target add $(MACOS_AARCH64)
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --release --target $(MACOS_AARCH64)
	@mkdir -p $(BUILD_DIR)/diskdb-$(VERSION)-macos-aarch64
	@cp target/$(MACOS_AARCH64)/release/diskdb $(BUILD_DIR)/diskdb-$(VERSION)-macos-aarch64/
	@cp README.md $(BUILD_DIR)/diskdb-$(VERSION)-macos-aarch64/
	@tar -czf $(BUILD_DIR)/diskdb-$(VERSION)-macos-aarch64.tar.gz -C $(BUILD_DIR) diskdb-$(VERSION)-macos-aarch64

release-linux-x64:
	@echo "Building for Linux x64..."
	@mkdir -p $(BUILD_DIR)
	rustup target add $(LINUX_X86_64)
	cargo build --release --target $(LINUX_X86_64)
	@mkdir -p $(BUILD_DIR)/diskdb-$(VERSION)-linux-x86_64
	@cp target/$(LINUX_X86_64)/release/diskdb $(BUILD_DIR)/diskdb-$(VERSION)-linux-x86_64/
	@cp README.md $(BUILD_DIR)/diskdb-$(VERSION)-linux-x86_64/
	@tar -czf $(BUILD_DIR)/diskdb-$(VERSION)-linux-x86_64.tar.gz -C $(BUILD_DIR) diskdb-$(VERSION)-linux-x86_64

release-linux-arm:
	@echo "Building for Linux ARM..."
	@mkdir -p $(BUILD_DIR)
	rustup target add $(LINUX_AARCH64)
	cargo build --release --target $(LINUX_AARCH64)
	@mkdir -p $(BUILD_DIR)/diskdb-$(VERSION)-linux-aarch64
	@cp target/$(LINUX_AARCH64)/release/diskdb $(BUILD_DIR)/diskdb-$(VERSION)-linux-aarch64/
	@cp README.md $(BUILD_DIR)/diskdb-$(VERSION)-linux-aarch64/
	@tar -czf $(BUILD_DIR)/diskdb-$(VERSION)-linux-aarch64.tar.gz -C $(BUILD_DIR) diskdb-$(VERSION)-linux-aarch64

# Build all platforms
release-all: release-macos-intel release-macos-arm release-linux-x64

# Format code
fmt:
	cargo fmt --all

# Run Clippy (Rust linter)
clippy:
	cargo clippy --all-targets --all-features

# Run tests
test:
	cargo test

# Connection test
connection-test:
	python clients/connection_test.py

# Python client test
test-python:
	cd clients && python test_all_datatypes.py

# Clean build files
clean:
	cargo clean
	rm -rf $(BUILD_DIR)
	rm -rf clients/python/build
	rm -rf clients/python/dist
	rm -rf clients/python/*.egg-info

# Install required Rust targets
install-targets:
	rustup target add $(LINUX_X86_64) $(LINUX_AARCH64) $(MACOS_X86_64) $(MACOS_AARCH64)

# Package Python client
package-python:
	@echo "Building Python package..."
	@cd clients/python && python setup.py sdist bdist_wheel

# Docker build
docker-build:
	docker build -t diskdb:$(VERSION) .

# Docker run
docker-run:
	docker run -p 6380:6380 --rm diskdb:$(VERSION)

# Show help
help:
	@echo "DiskDB Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make build              - Build DiskDB for development"
	@echo "  make release-all        - Build releases for all platforms"
	@echo "  make release-macos-intel - Build for macOS Intel"
	@echo "  make release-macos-arm  - Build for macOS Apple Silicon"
	@echo "  make release-linux-x64  - Build for Linux x64"
	@echo "  make test               - Run tests"
	@echo "  make test-python        - Run Python client tests"
	@echo "  make package-python     - Build Python package"
	@echo "  make clean              - Clean build artifacts"
	@echo "  make docker-build       - Build Docker image"
	@echo ""