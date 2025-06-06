#!/usr/bin/env python3
"""
Test script for DiskDB Python client
Demonstrates all supported operations
"""

import sys
import time
import json

# Add the Python client to path
sys.path.append('clients/python')

from diskdb import DiskDB, DiskDBError, TypeMismatchError

def print_section(title):
    """Print a formatted section header"""
    print(f"\n{'='*60}")
    print(f" {title}")
    print(f"{'='*60}")

def test_connection():
    """Test basic connection to DiskDB"""
    print_section("Testing Connection")
    
    try:
        db = DiskDB(host='localhost', port=6380)
        db.set("test_connection", "success")
        result = db.get("test_connection")
        print(f"✅ Connection successful! Test value: {result}")
        db.close()
        return True
    except Exception as e:
        print(f"❌ Connection failed: {e}")
        print("\nMake sure DiskDB is running on port 6380:")
        print("  ./target/release/diskdb")
        return False

def test_string_operations():
    """Test string operations"""
    print_section("String Operations")
    
    with DiskDB() as db:
        # SET and GET
        db.set("name", "DiskDB")
        print(f"SET name = 'DiskDB'")
        print(f"GET name = '{db.get('name')}'")
        
        # INCR, DECR, INCRBY
        db.set("counter", "10")
        print(f"\nInitial counter: {db.get('counter')}")
        print(f"INCR counter: {db.incr('counter')}")
        print(f"DECR counter: {db.decr('counter')}")
        print(f"INCRBY counter 5: {db.incrby('counter', 5)}")
        
        # APPEND
        db.set("message", "Hello")
        print(f"\nInitial message: '{db.get('message')}'")
        length = db.append("message", " World!")
        print(f"APPEND ' World!': '{db.get('message')}' (length: {length})")
        
        # Non-existent key
        print(f"\nGET non_existent: {db.get('non_existent')}")

def test_list_operations():
    """Test list operations"""
    print_section("List Operations")
    
    with DiskDB() as db:
        db.delete("mylist")
        
        # LPUSH and RPUSH
        print("LPUSH mylist 'first':", db.lpush("mylist", "first"))
        print("RPUSH mylist 'second':", db.rpush("mylist", "second"))
        print("LPUSH mylist 'zero':", db.lpush("mylist", "zero"))
        
        # LRANGE
        print("\nLRANGE mylist 0 -1:", db.lrange("mylist", 0, -1))
        
        # LPOP and RPOP
        print("\nLPOP mylist:", db.lpop("mylist"))
        print("RPOP mylist:", db.rpop("mylist"))
        print("Final list:", db.lrange("mylist", 0, -1))
        
        # LLEN
        print("LLEN mylist:", db.llen("mylist"))

def test_set_operations():
    """Test set operations"""
    print_section("Set Operations")
    
    with DiskDB() as db:
        db.delete("myset")
        
        # SADD
        print("SADD myset 'apple':", db.sadd("myset", "apple"))
        print("SADD myset 'banana' 'orange':", db.sadd("myset", "banana", "orange"))
        print("SADD myset 'apple' (duplicate):", db.sadd("myset", "apple"))
        
        # SMEMBERS
        members = db.smembers("myset")
        print("\nSMEMBERS myset:", sorted(members))
        
        # SISMEMBER
        print("\nSISMEMBER myset 'apple':", db.sismember("myset", "apple"))
        print("SISMEMBER myset 'grape':", db.sismember("myset", "grape"))
        
        # SREM
        print("\nSREM myset 'banana':", db.srem("myset", "banana"))
        print("Final set:", sorted(db.smembers("myset")))
        
        # SCARD
        print("SCARD myset:", db.scard("myset"))

def test_hash_operations():
    """Test hash operations"""
    print_section("Hash Operations")
    
    with DiskDB() as db:
        db.delete("user:1001")
        
        # HSET
        print("HSET user:1001 'name' 'Alice':", db.hset("user:1001", "name", "Alice"))
        print("HSET user:1001 'age' '30':", db.hset("user:1001", "age", "30"))
        print("HSET user:1001 'city' 'NYC':", db.hset("user:1001", "city", "NYC"))
        
        # HGET
        print("\nHGET user:1001 'name':", db.hget("user:1001", "name"))
        print("HGET user:1001 'age':", db.hget("user:1001", "age"))
        
        # HGETALL
        print("\nHGETALL user:1001:", db.hgetall("user:1001"))
        
        # HEXISTS
        print("\nHEXISTS user:1001 'name':", db.hexists("user:1001", "name"))
        print("HEXISTS user:1001 'email':", db.hexists("user:1001", "email"))
        
        # HDEL
        print("\nHDEL user:1001 'age':", db.hdel("user:1001", "age"))
        print("Final hash:", db.hgetall("user:1001"))

def test_sorted_set_operations():
    """Test sorted set operations"""
    print_section("Sorted Set Operations")
    
    with DiskDB() as db:
        db.delete("leaderboard")
        
        # ZADD
        scores = {"alice": 100, "bob": 85, "charlie": 92, "david": 88}
        print("ZADD leaderboard (multiple):", db.zadd("leaderboard", scores))
        
        # ZRANGE
        print("\nZRANGE leaderboard 0 -1:")
        for member in db.zrange("leaderboard", 0, -1):
            print(f"  {member}")
        
        print("\nZRANGE with scores:")
        for member, score in db.zrange("leaderboard", 0, -1, withscores=True):
            print(f"  {member}: {score}")
        
        # ZSCORE
        print("\nZSCORE leaderboard 'alice':", db.zscore("leaderboard", "alice"))
        print("ZSCORE leaderboard 'unknown':", db.zscore("leaderboard", "unknown"))
        
        # ZREM
        print("\nZREM leaderboard 'bob':", db.zrem("leaderboard", "bob"))
        
        # ZCARD
        print("ZCARD leaderboard:", db.zcard("leaderboard"))

def test_json_operations():
    """Test JSON operations"""
    print_section("JSON Operations")
    
    with DiskDB() as db:
        db.delete("user:json")
        
        # JSON.SET
        user_data = {
            "name": "John Doe",
            "age": 35,
            "email": "john@example.com",
            "address": {
                "street": "123 Main St",
                "city": "San Francisco",
                "zip": "94105"
            },
            "hobbies": ["coding", "reading", "hiking"]
        }
        
        print("JSON.SET user:json '$' (full document)")
        db.json_set("user:json", "$", user_data)
        
        # JSON.GET
        print("\nJSON.GET user:json '$':")
        retrieved = db.json_get("user:json", "$")
        print(json.dumps(retrieved, indent=2))
        
        print("\nJSON.GET user:json '$.address.city':", db.json_get("user:json", "$.address.city"))
        print("JSON.GET user:json '$.hobbies':", db.json_get("user:json", "$.hobbies"))
        
        # JSON.DEL
        print("\nJSON.DEL user:json '$':", db.json_del("user:json", "$"))
        print("After deletion:", db.json_get("user:json", "$"))

def test_stream_operations():
    """Test stream operations"""
    print_section("Stream Operations")
    
    with DiskDB() as db:
        db.delete("events:stream")
        
        # XADD
        print("Adding events to stream...")
        id1 = db.xadd("events:stream", {"action": "login", "user": "alice", "ip": "192.168.1.1"})
        print(f"Event 1 ID: {id1}")
        
        time.sleep(0.1)  # Small delay to ensure different timestamps
        
        id2 = db.xadd("events:stream", {"action": "purchase", "user": "alice", "amount": "99.99"})
        print(f"Event 2 ID: {id2}")
        
        id3 = db.xadd("events:stream", {"action": "logout", "user": "alice"})
        print(f"Event 3 ID: {id3}")
        
        # XLEN
        print(f"\nXLEN events:stream: {db.xlen('events:stream')}")
        
        # XRANGE
        print("\nXRANGE events:stream - +:")
        events = db.xrange("events:stream", "-", "+")
        for event in events:
            print(f"  ID: {event['id']}")
            for key, value in event['fields'].items():
                print(f"    {key}: {value}")

def test_utility_operations():
    """Test utility operations"""
    print_section("Utility Operations")
    
    with DiskDB() as db:
        # Prepare test data
        db.set("str_key", "value")
        db.lpush("list_key", "item")
        db.sadd("set_key", "member")
        db.hset("hash_key", "field", "value")
        db.zadd("zset_key", {"member": 1.0})
        
        # TYPE
        print("TYPE commands:")
        for key in ["str_key", "list_key", "set_key", "hash_key", "zset_key", "non_existent"]:
            print(f"  TYPE {key}: {db.type(key)}")
        
        # EXISTS
        print("\nEXISTS commands:")
        print(f"  EXISTS str_key: {db.exists('str_key')}")
        print(f"  EXISTS str_key list_key: {db.exists('str_key', 'list_key')}")
        print(f"  EXISTS non_existent: {db.exists('non_existent')}")
        
        # DELETE
        print("\nDELETE commands:")
        print(f"  DELETE str_key: {db.delete('str_key')}")
        print(f"  DELETE list_key set_key: {db.delete('list_key', 'set_key')}")
        print(f"  EXISTS str_key after delete: {db.exists('str_key')}")
        
        # Server status check (alternative to INFO)
        print("\nServer status check:")
        try:
            # Test connectivity with a simple operation
            test_key = "_server_test_"
            db.set(test_key, "ok")
            if db.get(test_key) == "ok":
                print("  ✅ Server is responsive")
                db.delete(test_key)
            else:
                print("  ⚠️ Server responded but with unexpected result")
        except Exception as e:
            print(f"  ❌ Server error: {e}")

def test_error_handling():
    """Test error handling"""
    print_section("Error Handling")
    
    with DiskDB() as db:
        # Type mismatch
        try:
            db.set("string_key", "value")
            db.lpush("string_key", "item")  # Should fail - wrong type
        except TypeMismatchError as e:
            print(f"✅ Caught expected TypeMismatchError: {e}")
        
        # Invalid number for INCR
        try:
            db.set("not_a_number", "abc")
            db.incr("not_a_number")  # Should fail - not a number
        except DiskDBError as e:
            print(f"✅ Caught expected DiskDBError: {e}")

def test_performance():
    """Quick performance test"""
    print_section("Performance Test")
    
    with DiskDB() as db:
        # Write performance
        num_ops = 1000
        start = time.time()
        
        for i in range(num_ops):
            db.set(f"perf_key_{i}", f"value_{i}")
        
        write_time = time.time() - start
        write_ops_per_sec = num_ops / write_time
        
        print(f"Write Performance:")
        print(f"  {num_ops} operations in {write_time:.3f}s")
        print(f"  {write_ops_per_sec:,.0f} ops/sec")
        
        # Read performance
        start = time.time()
        
        for i in range(num_ops):
            db.get(f"perf_key_{i}")
        
        read_time = time.time() - start
        read_ops_per_sec = num_ops / read_time
        
        print(f"\nRead Performance:")
        print(f"  {num_ops} operations in {read_time:.3f}s")
        print(f"  {read_ops_per_sec:,.0f} ops/sec")
        
        # Cleanup
        for i in range(num_ops):
            db.delete(f"perf_key_{i}")

def main():
    """Run all tests"""
    print("DiskDB Python Client Test Suite")
    print("==============================")
    
    # First test connection
    if not test_connection():
        return
    
    # Run all tests
    tests = [
        test_string_operations,
        test_list_operations,
        test_set_operations,
        test_hash_operations,
        test_sorted_set_operations,
        test_json_operations,
        test_stream_operations,
        test_utility_operations,
        test_error_handling,
        test_performance
    ]
    
    for test in tests:
        try:
            test()
        except Exception as e:
            print(f"\n❌ Test failed: {e}")
            import traceback
            traceback.print_exc()
    
    print("\n" + "="*60)
    print(" All tests completed!")
    print("="*60)

if __name__ == "__main__":
    main()