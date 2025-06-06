#!/usr/bin/env python3
"""
Test suite for DiskDB Python package
"""

import sys
import pytest
from diskdb import DiskDB, DiskDBError, TypeMismatchError

# Test if server is running
try:
    test_db = DiskDB()
    test_db.set("test", "test")
    test_db.close()
except:
    print("ERROR: DiskDB server is not running on localhost:6380")
    print("Please start the server with: cargo run --release")
    sys.exit(1)


class TestDiskDBStrings:
    """Test string operations."""
    
    def setup_method(self):
        self.db = DiskDB()
        self.db.delete("test_key", "counter", "msg")
    
    def teardown_method(self):
        self.db.close()
    
    def test_set_get(self):
        assert self.db.set("test_key", "test_value") is True
        assert self.db.get("test_key") == "test_value"
        assert self.db.get("nonexistent") is None
    
    def test_incr_decr(self):
        self.db.set("counter", "10")
        assert self.db.incr("counter") == 11
        assert self.db.decr("counter") == 10
        assert self.db.incrby("counter", 5) == 15
    
    def test_append(self):
        self.db.set("msg", "Hello")
        length = self.db.append("msg", "World")
        assert length == 10
        assert self.db.get("msg") == "HelloWorld"


class TestDiskDBLists:
    """Test list operations."""
    
    def setup_method(self):
        self.db = DiskDB()
        self.db.delete("mylist")
    
    def teardown_method(self):
        self.db.close()
    
    def test_push_pop(self):
        assert self.db.lpush("mylist", "a") == 1
        assert self.db.lpush("mylist", "b", "c") == 3
        assert self.db.rpush("mylist", "d") == 4
        
        assert self.db.lpop("mylist") == "c"
        assert self.db.rpop("mylist") == "d"
        assert self.db.llen("mylist") == 2
    
    def test_lrange(self):
        self.db.rpush("mylist", "1", "2", "3", "4", "5")
        
        assert self.db.lrange("mylist", 0, -1) == ["1", "2", "3", "4", "5"]
        assert self.db.lrange("mylist", 1, 3) == ["2", "3", "4"]
        assert self.db.lrange("mylist", -2, -1) == ["4", "5"]


class TestDiskDBSets:
    """Test set operations."""
    
    def setup_method(self):
        self.db = DiskDB()
        self.db.delete("myset")
    
    def teardown_method(self):
        self.db.close()
    
    def test_add_remove(self):
        assert self.db.sadd("myset", "a") == 1
        assert self.db.sadd("myset", "b", "c") == 2
        assert self.db.sadd("myset", "a") == 0  # Already exists
        
        assert self.db.scard("myset") == 3
        assert self.db.sismember("myset", "a") is True
        assert self.db.sismember("myset", "z") is False
        
        assert self.db.srem("myset", "a") == 1
        assert self.db.scard("myset") == 2
    
    def test_smembers(self):
        self.db.sadd("myset", "apple", "banana", "orange")
        members = self.db.smembers("myset")
        
        assert isinstance(members, set)
        assert len(members) == 3
        assert "apple" in members
        assert "banana" in members
        assert "orange" in members


class TestDiskDBHashes:
    """Test hash operations."""
    
    def setup_method(self):
        self.db = DiskDB()
        self.db.delete("myhash")
    
    def teardown_method(self):
        self.db.close()
    
    def test_hash_operations(self):
        assert self.db.hset("myhash", "field1", "value1") == 1
        assert self.db.hset("myhash", "field2", "value2") == 1
        assert self.db.hset("myhash", "field1", "updated") == 0  # Update
        
        assert self.db.hget("myhash", "field1") == "updated"
        assert self.db.hget("myhash", "nonexistent") is None
        
        assert self.db.hexists("myhash", "field1") is True
        assert self.db.hexists("myhash", "nonexistent") is False
        
        assert self.db.hdel("myhash", "field1") == 1
        assert self.db.hexists("myhash", "field1") is False
    
    def test_hgetall(self):
        self.db.hset("myhash", "name", "Alice")
        self.db.hset("myhash", "age", "30")
        self.db.hset("myhash", "city", "NYC")
        
        all_fields = self.db.hgetall("myhash")
        assert all_fields == {"name": "Alice", "age": "30", "city": "NYC"}


class TestDiskDBSortedSets:
    """Test sorted set operations."""
    
    def setup_method(self):
        self.db = DiskDB()
        self.db.delete("myzset")
    
    def teardown_method(self):
        self.db.close()
    
    def test_sorted_set_operations(self):
        assert self.db.zadd("myzset", {"alice": 100, "bob": 90, "charlie": 95}) == 3
        assert self.db.zcard("myzset") == 3
        
        assert self.db.zscore("myzset", "alice") == 100.0
        assert self.db.zscore("myzset", "unknown") is None
        
        # Range without scores
        members = self.db.zrange("myzset", 0, -1)
        assert members == ["bob", "charlie", "alice"]
        
        # Range with scores
        with_scores = self.db.zrange("myzset", 0, -1, withscores=True)
        assert with_scores == [("bob", 90.0), ("charlie", 95.0), ("alice", 100.0)]
        
        assert self.db.zrem("myzset", "bob") == 1
        assert self.db.zcard("myzset") == 2


class TestDiskDBJSON:
    """Test JSON operations."""
    
    def setup_method(self):
        self.db = DiskDB()
        self.db.delete("myjson")
    
    def teardown_method(self):
        self.db.close()
    
    def test_json_operations(self):
        data = {
            "name": "Alice",
            "age": 30,
            "hobbies": ["reading", "coding"],
            "address": {
                "city": "NYC",
                "zip": "10001"
            }
        }
        
        assert self.db.json_set("myjson", "$", data) is True
        
        # Get full document
        retrieved = self.db.json_get("myjson", "$")
        assert retrieved["name"] == "Alice"
        assert retrieved["age"] == 30
        assert retrieved["hobbies"] == ["reading", "coding"]
        
        # Delete and verify
        assert self.db.json_del("myjson", "$") == 1
        assert self.db.json_get("myjson", "$") is None


class TestDiskDBStreams:
    """Test stream operations."""
    
    def setup_method(self):
        self.db = DiskDB()
        self.db.delete("mystream")
    
    def teardown_method(self):
        self.db.close()
    
    def test_stream_operations(self):
        # Add entries
        id1 = self.db.xadd("mystream", {"field1": "value1", "field2": "value2"})
        assert "-" in id1  # Auto-generated ID
        
        id2 = self.db.xadd("mystream", {"field3": "value3"})
        assert "-" in id2
        
        # Check length
        assert self.db.xlen("mystream") == 2
        
        # Range query
        entries = self.db.xrange("mystream", "-", "+")
        assert len(entries) >= 2


class TestDiskDBUtility:
    """Test utility operations."""
    
    def setup_method(self):
        self.db = DiskDB()
        self.db.delete("key1", "key2", "key3")
    
    def teardown_method(self):
        self.db.close()
    
    def test_type_command(self):
        self.db.set("key1", "value")
        self.db.lpush("key2", "item")
        self.db.sadd("key3", "member")
        
        assert self.db.type("key1") == "string"
        assert self.db.type("key2") == "list"
        assert self.db.type("key3") == "set"
        assert self.db.type("nonexistent") == "none"
    
    def test_exists_delete(self):
        self.db.set("key1", "value1")
        self.db.set("key2", "value2")
        
        assert self.db.exists("key1") == 1
        assert self.db.exists("key1", "key2") == 2
        assert self.db.exists("key1", "key2", "nonexistent") == 2
        
        assert self.db.delete("key1") == 1
        assert self.db.exists("key1") == 0
        assert self.db.delete("key2", "nonexistent") == 1


class TestDiskDBErrors:
    """Test error handling."""
    
    def setup_method(self):
        self.db = DiskDB()
        self.db.delete("mykey")
    
    def teardown_method(self):
        self.db.close()
    
    def test_type_mismatch(self):
        self.db.set("mykey", "string_value")
        
        # Try list operation on string
        with pytest.raises(TypeMismatchError):
            self.db.lpush("mykey", "item")
        
        # Try to increment non-numeric
        self.db.set("mykey", "not_a_number")
        with pytest.raises(DiskDBError):
            self.db.incr("mykey")


def test_context_manager():
    """Test context manager support."""
    with DiskDB() as db:
        db.set("test", "value")
        assert db.get("test") == "value"
    
    # Connection should be closed
    # Attempting to use would require reconnection


if __name__ == "__main__":
    # Run basic tests if pytest not available
    print("Running basic DiskDB tests...")
    
    db = DiskDB()
    
    # Test strings
    db.set("test", "hello")
    assert db.get("test") == "hello"
    print("✓ String operations")
    
    # Test lists
    db.delete("list")
    db.lpush("list", "a", "b")
    assert db.lrange("list", 0, -1) == ["b", "a"]
    print("✓ List operations")
    
    # Test sets
    db.delete("set")
    db.sadd("set", "x", "y")
    assert db.scard("set") == 2
    print("✓ Set operations")
    
    # Test JSON
    db.delete("json")
    db.json_set("json", "$", {"key": "value"})
    assert db.json_get("json", "$")["key"] == "value"
    print("✓ JSON operations")
    
    db.close()
    print("\nAll basic tests passed!")
    print("\nFor comprehensive testing, install pytest and run: pytest test_diskdb.py")