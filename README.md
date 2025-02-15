# DiskDB

DiskDB is an emerging database system designed to efficiently store data on disk while ensuring data consistency, thread safety, and high concurrency.

**Project Status**: This project is a work in progress and is not yet production-ready. We welcome contributions from the community to help drive its development forward.

## Features

- **Persistent Storage**: Data is saved to disk to prevent loss.
- **Data Consistency**: Ensures ACID properties for reliable transactions.
- **Thread-Safe**: Supports safe access from multiple threads.
- **High Concurrency**: Optimized for high-performance read and write operations.

## Installation

Before building the project, ensure that the required system dependencies are installed.

### macOS

```sh
brew install rocksdb cmake snappy lz4 zstd llvm@14
export PATH="/opt/homebrew/opt/llvm@14/bin:$PATH"
clang --version # Should return version 14
```

Then, build and run the project using Cargo:

```sh
cargo run
# or
cargo build
```

## Usage

Here's an example of how to use DiskDB in your project:

```rust
use diskdb::Database;

fn main() {
    let db = Database::new("data.db");
    db.insert("key", "value");
    let value = db.get("key");
    println!("Value: {:?}", value);
}
```

## Contributing

We welcome contributions from developers of all experience levels. To get started:

1. **Fork** the repository on GitHub.
2. **Clone** your fork to your local machine.
3. **Create a branch** for your feature or bug fix.
4. **Make your changes** with clear and concise commit messages.
5. **Push** your changes to your fork.
6. **Submit a pull request** to the main repository.

Please refer to our [CONTRIBUTING.md](CONTRIBUTING.md) file for more detailed guidelines.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
