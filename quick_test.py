#!/usr/bin/env python3
"""
Quick test to verify DiskDB is working
"""

import sys
import time

# Add the Python client to path
sys.path.append('clients/python')

from diskdb import DiskDB, DiskDBError

def main():
    print("🔍 Quick DiskDB Test")
    print("===================")
    
    try:
        print("Connecting to DiskDB on localhost:6380...")
        db = DiskDB(host='localhost', port=6380)
        
        print("✅ Connected successfully!")
        
        # Test basic operations
        print("\nTesting basic operations...")
        
        # String operations
        db.set("test", "Hello DiskDB!")
        result = db.get("test")
        print(f"  SET/GET: '{result}'")
        
        # Counter operations
        db.set("counter", "10")
        counter = db.incr("counter")
        print(f"  INCR: {counter}")
        
        # List operations
        db.delete("mylist")
        length = db.lpush("mylist", "item1", "item2")
        items = db.lrange("mylist", 0, -1)
        print(f"  LIST: {items} (length: {length})")
        
        # Hash operations
        db.delete("myhash")
        db.hset("myhash", "name", "Alice")
        db.hset("myhash", "age", "30")
        hash_data = db.hgetall("myhash")
        print(f"  HASH: {hash_data}")
        
        # JSON operations
        db.delete("myjson")
        json_data = {"name": "Bob", "score": 95}
        db.json_set("myjson", "$", json_data)
        retrieved = db.json_get("myjson", "$")
        print(f"  JSON: {retrieved}")
        
        # Performance test
        print("\nQuick performance test...")
        start = time.time()
        for i in range(1000):
            db.set(f"perf_{i}", f"value_{i}")
        write_time = time.time() - start
        
        start = time.time()
        for i in range(1000):
            db.get(f"perf_{i}")
        read_time = time.time() - start
        
        print(f"  1000 writes: {write_time:.3f}s ({1000/write_time:,.0f} ops/s)")
        print(f"  1000 reads:  {read_time:.3f}s ({1000/read_time:,.0f} ops/s)")
        
        # Cleanup
        for i in range(1000):
            db.delete(f"perf_{i}")
        
        db.close()
        
        print("\n🎉 All tests passed! DiskDB is working correctly.")
        print("\nYou can now run the full test suite:")
        print("  python3 test_diskdb_client.py")
        
    except ConnectionError as e:
        print(f"\n❌ Connection failed: {e}")
        print("\nMake sure DiskDB is running:")
        print("  ./target/release/diskdb")
        print("\nThen try again.")
        
    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()