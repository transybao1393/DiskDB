# DiskDB

DiskDB is a new database designed to efficiently store data on disk while ensuring data consistency, thread safety, and high concurrency.

## Features

- **Persistent Storage**: Data is saved to disk to prevent data loss.
- **Data Consistency**: Ensures ACID properties for reliable transactions.
- **Thread-Safe**: Supports safe access from multiple threads.
- **Concurrency**: Optimized for high-performance read and write operations.

## Installation

```sh
cargo build
```

## Usage

```rust
// Example usage of DiskDB
use diskdb::Database;

fn main() {
    let db = Database::new("data.db");
    db.insert("key", "value");
    let value = db.get("key");
    println!("Value: {:?}", value);
}
```

## License

This project is licensed under the MIT License.

