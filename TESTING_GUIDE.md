# DiskDB Testing Guide

This guide will help you test DiskDB with the Python client on macOS (Apple Silicon).

## Prerequisites

1. **Build Complete**: DiskDB has been built successfully
   - Binary location: `./target/release/diskdb`
   - Architecture: arm64 (Apple Silicon)

2. **Python Client**: Located in `clients/python/`
   - No external dependencies required
   - Pure Python implementation

## Step 1: Start DiskDB Server

Open a terminal and run:

```bash
# From the DiskDB project root directory
./target/release/diskdb
```

You should see output like:
```
Starting DiskDB server on 127.0.0.1:6380
Server is ready to accept connections
```

Keep this terminal open - the server needs to stay running.

## Step 2: Quick Test

In a **new terminal**, run a quick test first:

```bash
# From the DiskDB project root directory
python3 quick_test.py
```

This will verify basic connectivity and functionality.

## Step 3: Run the Full Test Suite

If the quick test passes, run the comprehensive test:

```bash
# From the DiskDB project root directory
python3 test_diskdb_client.py
```

This will test all DiskDB features including:
- String operations (SET, GET, INCR, APPEND)
- List operations (LPUSH, RPUSH, LPOP, LRANGE)
- Set operations (SADD, SREM, SMEMBERS)
- Hash operations (HSET, HGET, HGETALL)
- Sorted set operations (ZADD, ZRANGE, ZSCORE)
- JSON document operations
- Stream operations (event logging)
- Error handling
- Performance benchmarks

## Step 4: Interactive Testing

You can also test DiskDB interactively:

```python
# Start Python in the project directory
python3

# Import the client
import sys
sys.path.append('clients/python')
from diskdb import DiskDB

# Connect to DiskDB
db = DiskDB(host='localhost', port=6380)

# Try some operations
db.set("hello", "world")
print(db.get("hello"))  # Output: world

db.lpush("mylist", "a", "b", "c")
print(db.lrange("mylist", 0, -1))  # Output: ['c', 'b', 'a']

# Don't forget to close
db.close()
```

## Step 5: Run Individual Tests

Test specific functionality:

```bash
# Test Redis compatibility
python3 tests/redis_diskdb_comparison.py

# Test all data types
python3 clients/test_all_datatypes.py
```

## Step 6: Performance Testing

For quick performance comparison:

```python
from diskdb import DiskDB
import time

db = DiskDB()

# Test write speed
start = time.time()
for i in range(10000):
    db.set(f"key_{i}", f"value_{i}")
print(f"10K writes: {time.time() - start:.2f}s")

# Test read speed
start = time.time()
for i in range(10000):
    db.get(f"key_{i}")
print(f"10K reads: {time.time() - start:.2f}s")
```

## Common Issues and Solutions

### Issue: Connection Refused
```
ConnectionError: Cannot connect to DiskDB on port 6380
```
**Solution**: Make sure DiskDB server is running (`./target/release/diskdb`)

### Issue: Module Not Found
```
ModuleNotFoundError: No module named 'diskdb'
```
**Solution**: Run from project root or adjust Python path:
```python
import sys
sys.path.append('/path/to/DiskDB/clients/python')
```

### Issue: Type Mismatch Error
```
TypeMismatchError: WRONGTYPE Operation against a key holding the wrong kind of value
```
**Solution**: This is expected behavior. Delete the key first or use correct operations for the data type.

## Advanced Usage

### 1. Using Context Manager
```python
with DiskDB() as db:
    db.set("auto", "cleanup")
    # Connection automatically closed
```

### 2. Custom Host/Port
```python
db = DiskDB(host='192.168.1.100', port=6380)
```

### 3. Pipeline Simulation
```python
with db.pipeline() as pipe:
    pipe.set("k1", "v1")
    pipe.set("k2", "v2")
    pipe.incr("counter")
    # All executed together
```

### 4. JSON Documents
```python
user = {
    "name": "Alice",
    "email": "alice@example.com",
    "preferences": {
        "theme": "dark",
        "notifications": True
    }
}

db.json_set("user:1", "$", user)
theme = db.json_get("user:1", "$.preferences.theme")
```

## Monitoring Performance

While testing, you can monitor DiskDB performance:

1. **Check memory usage**:
   ```bash
   ps aux | grep diskdb
   ```

2. **Monitor disk I/O**:
   ```bash
   iotop  # May need: brew install iotop
   ```

3. **Check database size**:
   ```bash
   du -sh diskdb/
   ```

## Next Steps

1. **Run Production Workload**: Test with your actual use case
2. **Benchmark Against Redis**: Use `tests/redis_diskdb_comparison.py`
3. **Test Persistence**: Restart DiskDB and verify data persists
4. **Load Testing**: Try concurrent clients
5. **Explore Features**: Test JSON and Stream operations

## Tips

- DiskDB automatically persists all data to disk
- Default port is 6380 (not 6379 like Redis)
- All data is stored in the `diskdb/` directory
- Use `db.info()` to get server statistics
- Type `db.` and press Tab in Python for command completion

## Summary

You now have:
1. ✅ DiskDB server built and ready
2. ✅ Python client available
3. ✅ Comprehensive test suite
4. ✅ Interactive testing environment

Start the server and run the tests to see DiskDB in action!