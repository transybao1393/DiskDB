# DiskDB Python Client

A fast, simple, and reliable Python client for DiskDB - the persistent key-value database with Redis-compatible API.

## Features

- 🚀 **High Performance** - Optimized for speed with minimal overhead
- 🔄 **Redis Compatible** - Drop-in replacement for Redis in many use cases  
- 📦 **All Data Types** - Strings, Lists, Sets, Hashes, Sorted Sets, JSON, Streams
- 🛡️ **Type Safe** - Comprehensive type hints for better IDE support
- 🔌 **Zero Dependencies** - Pure Python implementation
- 🎯 **Simple API** - Intuitive methods that just work

## Installation

```bash
pip install diskdb
```

## Quick Start

```python
from diskdb import DiskDB

# Connect to DiskDB server
db = DiskDB(host='localhost', port=6380)

# String operations
db.set('name', 'Alice')
name = db.get('name')  # 'Alice'

# List operations
db.lpush('tasks', 'task1', 'task2', 'task3')
task = db.rpop('tasks')  # 'task1'

# Set operations
db.sadd('tags', 'python', 'database', 'nosql')
has_python = db.sismember('tags', 'python')  # True

# Hash operations
db.hset('user:1', 'name', 'Bob')
db.hset('user:1', 'age', '30')
user = db.hgetall('user:1')  # {'name': 'Bob', 'age': '30'}

# JSON operations
user_data = {'name': 'Charlie', 'scores': [100, 95, 87]}
db.json_set('player:1', '$', user_data)
scores = db.json_get('player:1', '$.scores')  # [100, 95, 87]
```

## Data Types

### Strings
Basic key-value storage with atomic operations.

```python
# Basic operations
db.set('counter', '0')
db.incr('counter')  # 1
db.incrby('counter', 5)  # 6
db.append('message', ' World')  # Returns new length

# Get/Set with None handling
value = db.get('nonexistent')  # None
```

### Lists
Ordered collections with fast head/tail operations.

```python
# Push and pop
db.lpush('queue', 'job1', 'job2')  # Push to left
db.rpush('queue', 'job3')  # Push to right
job = db.lpop('queue')  # Pop from left
job = db.rpop('queue')  # Pop from right

# Range operations
items = db.lrange('queue', 0, -1)  # Get all items
length = db.llen('queue')  # Get length
```

### Sets
Unordered collections of unique elements.

```python
# Add and remove
db.sadd('skills', 'python', 'rust', 'go')
db.srem('skills', 'go')

# Set operations
members = db.smembers('skills')  # Get all members
count = db.scard('skills')  # Get cardinality
is_member = db.sismember('skills', 'python')  # Check membership
```

### Hashes
Field-value pairs within a key.

```python
# Set fields
db.hset('product:1', 'name', 'Laptop')
db.hset('product:1', 'price', '999')

# Get operations
name = db.hget('product:1', 'name')
all_fields = db.hgetall('product:1')
exists = db.hexists('product:1', 'price')

# Delete fields
db.hdel('product:1', 'old_field')
```

### Sorted Sets
Sets with scores for ordering.

```python
# Add with scores
db.zadd('leaderboard', {'alice': 100, 'bob': 95, 'charlie': 102})

# Range queries
top_players = db.zrange('leaderboard', 0, 2)  # Top 3 players
with_scores = db.zrange('leaderboard', 0, -1, withscores=True)

# Score operations
score = db.zscore('leaderboard', 'alice')  # Get score
db.zrem('leaderboard', 'bob')  # Remove member
```

### JSON
Native JSON support with path queries.

```python
# Store complex objects
profile = {
    'name': 'Alice',
    'age': 30,
    'interests': ['coding', 'music'],
    'address': {
        'city': 'NYC',
        'zip': '10001'
    }
}
db.json_set('user:profile', '$', profile)

# Query with paths
city = db.json_get('user:profile', '$.address.city')
interests = db.json_get('user:profile', '$.interests')

# Delete paths
db.json_del('user:profile', '$.address')
```

### Streams
Append-only logs for event streaming.

```python
# Add entries
entry_id = db.xadd('events', {'action': 'login', 'user': 'alice'})
entry_id = db.xadd('events', {'action': 'purchase', 'amount': '99.99'})

# Read entries
entries = db.xrange('events', '-', '+')  # All entries
recent = db.xrange('events', '-', '+', count=10)  # Last 10

# Stream info
length = db.xlen('events')
```

## Utility Operations

```python
# Key management
db.exists('key1', 'key2')  # Check multiple keys
db.delete('key1', 'key2')  # Delete multiple keys
key_type = db.type('mykey')  # Get key type

# Connection management
db.close()  # Close connection

# Context manager
with DiskDB() as db:
    db.set('key', 'value')
    # Connection automatically closed
```

## Error Handling

```python
from diskdb import DiskDB, DiskDBError, TypeMismatchError

try:
    db = DiskDB()
    db.set('mystring', 'value')
    db.lpush('mystring', 'item')  # Wrong type!
except TypeMismatchError as e:
    print(f"Type error: {e}")
except DiskDBError as e:
    print(f"Database error: {e}")
```

## Performance Tips

1. **Reuse Connections**: Create one client and reuse it
2. **Use Context Managers**: Ensures proper cleanup
3. **Batch Operations**: Use pipelines for multiple commands (coming soon)
4. **Choose Right Data Type**: Each type is optimized for specific use cases

## Requirements

- Python 3.7+
- DiskDB server running on accessible host

## Contributing

We welcome contributions! Please see our [Contributing Guide](https://github.com/transybao1393/DiskDB/blob/main/CONTRIBUTING.md).

## License

MIT License - see [LICENSE](LICENSE) for details.