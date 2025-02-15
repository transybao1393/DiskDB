# Define targets
TARGET_DIR = target/release
BINARY_NAME = my_binary

# Define cross-compilation targets
LINUX_X86_64 = x86_64-unknown-linux-gnu
WINDOWS_X86_64 = x86_64-pc-windows-gnu
MACOS_X86_64 = x86_64-apple-darwin
MACOS_AARCH64 = aarch64-apple-darwin

# Default build (release)
build:
	cargo build --release

# Debug build
debug:
	cargo build

# Debug run
debug-run:
	cargo run

# Cross-compile for Linux (x86_64)
linux:
	cargo build --release --target $(LINUX_X86_64)

# Cross-compile for Windows (x86_64)
windows:
	cargo build --release --target $(WINDOWS_X86_64)

# Cross-compile for macOS (x86_64)
macos:
	cargo build --release --target $(MACOS_X86_64)

# Cross-compile for macOS (ARM M1/M2)
macos-arm:
	cargo build --release --target $(MACOS_AARCH64)

# Format code
fmt:
	cargo fmt --all

# Run Clippy (Rust linter)
clippy:
	cargo clippy --all-targets --all-features

# Run tests
test:
	cargo test

# Strip binary to reduce size (Linux/macOS)
strip:
	strip $(TARGET_DIR)/$(BINARY_NAME) || true

# Clean build files
clean:
	cargo clean

# Install required Rust targets
install-targets:
	rustup target add $(LINUX_X86_64) $(WINDOWS_X86_64) $(MACOS_X86_64) $(MACOS_AARCH64)

# Build all targets
build-all: linux windows macos macos-arm

# Package binaries into a release folder
package:
	mkdir -p release
	cp $(TARGET_DIR)/$(BINARY_NAME) release/$(BINARY_NAME)-linux || true
	cp $(TARGET_DIR)/$(BINARY_NAME).exe release/$(BINARY_NAME)-windows.exe || true
	cp $(TARGET_DIR)/$(BINARY_NAME) release/$(BINARY_NAME)-macos || true

# Dockerfile buikd as diskdb image
docker-build:
	docker build -t diskdb .

# Dockerfile run as diskdb container at port 6380
docker-run:
	docker run -p 6380:6380 --rm diskdb

.PHONY: build debug linux windows macos macos-arm fmt clippy test strip clean install-targets build-all package