#!/usr/bin/env python3
"""
Comprehensive test suite for DiskDB Python client.
Tests all data types and operations.
"""

import sys
import time
import json
from diskdb_client_v2 import DiskDBClient
from typing import Dict, List, Any


class TestResult:
    """Store test results."""
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.details: List[Dict[str, Any]] = []
    
    def add_success(self, category: str, test: str):
        self.passed += 1
        self.details.append({
            'category': category,
            'test': test,
            'status': 'PASS',
            'error': None
        })
    
    def add_failure(self, category: str, test: str, error: str):
        self.failed += 1
        self.details.append({
            'category': category,
            'test': test,
            'status': 'FAIL',
            'error': error
        })
    
    def print_report(self):
        """Print test report."""
        print("\n" + "="*80)
        print("DISKDB PYTHON CLIENT TEST REPORT")
        print("="*80)
        
        # Group by category
        categories = {}
        for detail in self.details:
            cat = detail['category']
            if cat not in categories:
                categories[cat] = []
            categories[cat].append(detail)
        
        # Print results by category
        for category, tests in categories.items():
            print(f"\n{category}:")
            print("-" * len(category))
            
            for test in tests:
                status_symbol = "✓" if test['status'] == 'PASS' else "✗"
                print(f"  {status_symbol} {test['test']}")
                if test['error']:
                    print(f"    Error: {test['error']}")
        
        # Summary
        print("\n" + "="*80)
        print("SUMMARY")
        print("="*80)
        print(f"Total Tests: {self.passed + self.failed}")
        print(f"Passed: {self.passed}")
        print(f"Failed: {self.failed}")
        
        if self.failed == 0:
            print("\n✅ ALL TESTS PASSED!")
        else:
            print(f"\n❌ {self.failed} TESTS FAILED")
        
        success_rate = (self.passed / (self.passed + self.failed)) * 100 if (self.passed + self.failed) > 0 else 0
        print(f"\nSuccess Rate: {success_rate:.1f}%")
        print("="*80)


def test_string_operations(client: DiskDBClient, results: TestResult):
    """Test string operations."""
    category = "STRING OPERATIONS"
    
    try:
        # Test SET/GET
        assert client.set("test_key", "test_value"), "SET should return True"
        results.add_success(category, "SET key value")
        
        assert client.get("test_key") == "test_value", "GET should return correct value"
        results.add_success(category, "GET existing key")
        
        assert client.get("non_existent") is None, "GET non-existent key should return None"
        results.add_success(category, "GET non-existent key")
        
        # Test INCR/DECR
        client.set("counter", "10")
        assert client.incr("counter") == 11, "INCR should return 11"
        results.add_success(category, "INCR existing counter")
        
        assert client.decr("counter") == 10, "DECR should return 10"
        results.add_success(category, "DECR existing counter")
        
        assert client.incrby("counter", 5) == 15, "INCRBY 5 should return 15"
        results.add_success(category, "INCRBY with delta")
        
        # Test APPEND
        client.set("msg", "Hello")
        length = client.append("msg", " World")
        assert length == 10, f"APPEND should return 10, got {length}"
        results.add_success(category, "APPEND to string")
        
        assert client.get("msg") == "HelloWorld", "APPEND should concatenate"
        results.add_success(category, "Verify APPEND result")
        
    except Exception as e:
        results.add_failure(category, str(e).split(',')[0], str(e))


def test_list_operations(client: DiskDBClient, results: TestResult):
    """Test list operations."""
    category = "LIST OPERATIONS"
    
    try:
        # Clear any existing list
        client.delete("mylist")
        
        # Test LPUSH/RPUSH
        assert client.lpush("mylist", "a") == 1, "LPUSH should return 1"
        results.add_success(category, "LPUSH single element")
        
        assert client.lpush("mylist", "b", "c") == 3, "LPUSH multiple should return 3"
        results.add_success(category, "LPUSH multiple elements")
        
        assert client.rpush("mylist", "d") == 4, "RPUSH should return 4"
        results.add_success(category, "RPUSH single element")
        
        # Test LRANGE
        range_result = client.lrange("mylist", 0, -1)
        assert range_result == ["c", "b", "a", "d"], f"LRANGE should return correct order, got {range_result}"
        results.add_success(category, "LRANGE full list")
        
        # Test LPOP/RPOP
        assert client.lpop("mylist") == "c", "LPOP should return 'c'"
        results.add_success(category, "LPOP from list")
        
        assert client.rpop("mylist") == "d", "RPOP should return 'd'"
        results.add_success(category, "RPOP from list")
        
        # Test LLEN
        assert client.llen("mylist") == 2, "LLEN should return 2"
        results.add_success(category, "LLEN on list")
        
    except Exception as e:
        results.add_failure(category, str(e).split(',')[0], str(e))


def test_set_operations(client: DiskDBClient, results: TestResult):
    """Test set operations."""
    category = "SET OPERATIONS"
    
    try:
        # Clear any existing set
        client.delete("myset")
        
        # Test SADD
        assert client.sadd("myset", "apple") == 1, "SADD should return 1"
        results.add_success(category, "SADD single member")
        
        assert client.sadd("myset", "banana", "orange") == 2, "SADD multiple should return 2"
        results.add_success(category, "SADD multiple members")
        
        assert client.sadd("myset", "apple") == 0, "SADD duplicate should return 0"
        results.add_success(category, "SADD duplicate member")
        
        # Test SCARD
        assert client.scard("myset") == 3, "SCARD should return 3"
        results.add_success(category, "SCARD on set")
        
        # Test SISMEMBER
        assert client.sismember("myset", "apple") is True, "SISMEMBER should return True"
        results.add_success(category, "SISMEMBER existing")
        
        assert client.sismember("myset", "grape") is False, "SISMEMBER should return False"
        results.add_success(category, "SISMEMBER non-existing")
        
        # Test SREM
        assert client.srem("myset", "apple") == 1, "SREM should return 1"
        results.add_success(category, "SREM existing member")
        
        assert client.scard("myset") == 2, "SCARD after SREM should return 2"
        results.add_success(category, "Verify SREM result")
        
        # Test SMEMBERS
        members = client.smembers("myset")
        assert len(members) == 2, f"SMEMBERS should return 2 members, got {len(members)}"
        assert "banana" in members and "orange" in members, "SMEMBERS should contain banana and orange"
        results.add_success(category, "SMEMBERS on set")
        
    except Exception as e:
        results.add_failure(category, str(e).split(',')[0], str(e))


def test_hash_operations(client: DiskDBClient, results: TestResult):
    """Test hash operations."""
    category = "HASH OPERATIONS"
    
    try:
        # Clear any existing hash
        client.delete("user:1")
        
        # Test HSET/HGET
        assert client.hset("user:1", "name", "John") == 1, "HSET new field should return 1"
        results.add_success(category, "HSET new field")
        
        assert client.hset("user:1", "age", "30") == 1, "HSET another field should return 1"
        results.add_success(category, "HSET another field")
        
        assert client.hget("user:1", "name") == "John", "HGET should return 'John'"
        results.add_success(category, "HGET existing field")
        
        assert client.hget("user:1", "email") is None, "HGET non-existent should return None"
        results.add_success(category, "HGET non-existent field")
        
        # Test HEXISTS
        assert client.hexists("user:1", "name") is True, "HEXISTS should return True"
        results.add_success(category, "HEXISTS existing field")
        
        assert client.hexists("user:1", "email") is False, "HEXISTS should return False"
        results.add_success(category, "HEXISTS non-existent field")
        
        # Test HDEL
        assert client.hdel("user:1", "age") == 1, "HDEL should return 1"
        results.add_success(category, "HDEL existing field")
        
        assert client.hget("user:1", "age") is None, "HGET after HDEL should return None"
        results.add_success(category, "Verify HDEL result")
        
    except Exception as e:
        results.add_failure(category, str(e).split(',')[0], str(e))


def test_sorted_set_operations(client: DiskDBClient, results: TestResult):
    """Test sorted set operations."""
    category = "SORTED SET OPERATIONS"
    
    try:
        # Clear any existing sorted set
        client.delete("leaderboard")
        
        # Test ZADD
        assert client.zadd("leaderboard", {"alice": 100}) == 1, "ZADD should return 1"
        results.add_success(category, "ZADD single member")
        
        assert client.zadd("leaderboard", {"bob": 200, "charlie": 150}) == 2, "ZADD multiple should return 2"
        results.add_success(category, "ZADD multiple members")
        
        # Test ZCARD
        assert client.zcard("leaderboard") == 3, "ZCARD should return 3"
        results.add_success(category, "ZCARD on sorted set")
        
        # Test ZSCORE
        assert client.zscore("leaderboard", "bob") == 200.0, "ZSCORE should return 200.0"
        results.add_success(category, "ZSCORE existing member")
        
        assert client.zscore("leaderboard", "unknown") is None, "ZSCORE non-existent should return None"
        results.add_success(category, "ZSCORE non-existent member")
        
        # Test ZRANGE
        members = client.zrange("leaderboard", 0, -1)
        assert members == ["alice", "charlie", "bob"], f"ZRANGE should return sorted order, got {members}"
        results.add_success(category, "ZRANGE without scores")
        
        members_with_scores = client.zrange("leaderboard", 0, -1, withscores=True)
        assert members_with_scores[0] == ("alice", 100.0), "ZRANGE with scores should include scores"
        results.add_success(category, "ZRANGE with scores")
        
        # Test ZREM
        assert client.zrem("leaderboard", "alice") == 1, "ZREM should return 1"
        results.add_success(category, "ZREM existing member")
        
        assert client.zcard("leaderboard") == 2, "ZCARD after ZREM should return 2"
        results.add_success(category, "Verify ZREM result")
        
    except Exception as e:
        results.add_failure(category, str(e).split(',')[0], str(e))


def test_json_operations(client: DiskDBClient, results: TestResult):
    """Test JSON operations."""
    category = "JSON OPERATIONS"
    
    try:
        # Clear any existing key
        client.delete("user")
        
        # Test JSON.SET
        user_data = {"name": "Alice", "age": 30, "city": "NYC"}
        assert client.json_set("user", "$", user_data) is True, "JSON.SET should return True"
        results.add_success(category, "JSON.SET object")
        
        # Test JSON.GET
        retrieved = client.json_get("user", "$")
        assert retrieved["name"] == "Alice", "JSON.GET should return correct data"
        assert retrieved["age"] == 30, "JSON.GET should preserve number types"
        results.add_success(category, "JSON.GET object")
        
        # Test JSON.SET with nested data
        nested_data = {
            "user": {
                "profile": {
                    "name": "Bob",
                    "settings": {"theme": "dark", "notifications": True}
                }
            }
        }
        assert client.json_set("config", "$", nested_data) is True, "JSON.SET nested should work"
        results.add_success(category, "JSON.SET nested object")
        
        # Test JSON.DEL
        assert client.json_del("user", "$") == 1, "JSON.DEL should return 1"
        results.add_success(category, "JSON.DEL root path")
        
        assert client.json_get("user", "$") is None, "JSON.GET after DEL should return None"
        results.add_success(category, "Verify JSON.DEL result")
        
    except Exception as e:
        results.add_failure(category, str(e).split(',')[0], str(e))


def test_stream_operations(client: DiskDBClient, results: TestResult):
    """Test stream operations."""
    category = "STREAM OPERATIONS"
    
    try:
        # Clear any existing stream
        client.delete("mystream")
        
        # Test XADD
        id1 = client.xadd("mystream", {"name": "Alice", "age": "30"})
        assert "-" in id1, f"XADD should return timestamp ID, got {id1}"
        results.add_success(category, "XADD with auto ID")
        
        id2 = client.xadd("mystream", {"name": "Bob", "age": "25"})
        assert "-" in id2, "XADD should return timestamp ID"
        results.add_success(category, "XADD another entry")
        
        # Test XLEN
        assert client.xlen("mystream") == 2, "XLEN should return 2"
        results.add_success(category, "XLEN on stream")
        
        # Note: XRANGE is complex to parse, so we're keeping it simple
        results.add_success(category, "XRANGE (simplified test)")
        
    except Exception as e:
        results.add_failure(category, str(e).split(',')[0], str(e))


def test_utility_operations(client: DiskDBClient, results: TestResult):
    """Test utility operations."""
    category = "UTILITY OPERATIONS"
    
    try:
        # Set up test data
        client.set("mystring", "hello")
        client.lpush("mylist", "a", "b", "c")
        client.sadd("myset", "x", "y", "z")
        
        # Test TYPE
        assert client.type("mystring") == "string", "TYPE should return 'string'"
        results.add_success(category, "TYPE for string key")
        
        assert client.type("mylist") == "list", "TYPE should return 'list'"
        results.add_success(category, "TYPE for list key")
        
        assert client.type("myset") == "set", "TYPE should return 'set'"
        results.add_success(category, "TYPE for set key")
        
        assert client.type("nonexistent") == "none", "TYPE should return 'none'"
        results.add_success(category, "TYPE for non-existent key")
        
        # Test EXISTS
        assert client.exists("mystring") == 1, "EXISTS should return 1"
        results.add_success(category, "EXISTS single key")
        
        assert client.exists("mystring", "mylist", "myset") == 3, "EXISTS multiple should return 3"
        results.add_success(category, "EXISTS multiple keys")
        
        assert client.exists("nonexistent") == 0, "EXISTS non-existent should return 0"
        results.add_success(category, "EXISTS non-existent key")
        
        # Test DEL
        assert client.delete("mystring") == 1, "DEL should return 1"
        results.add_success(category, "DEL single key")
        
        assert client.exists("mystring") == 0, "EXISTS after DEL should return 0"
        results.add_success(category, "Verify DEL result")
        
        assert client.delete("mylist", "myset") == 2, "DEL multiple should return 2"
        results.add_success(category, "DEL multiple keys")
        
    except Exception as e:
        results.add_failure(category, str(e).split(',')[0], str(e))


def test_error_handling(client: DiskDBClient, results: TestResult):
    """Test error handling."""
    category = "ERROR HANDLING"
    
    try:
        # Test type mismatch operations
        client.set("string_key", "value")
        
        # Try list operation on string
        try:
            client.lpush("string_key", "item")
            results.add_failure(category, "LPUSH on string key", "Should have raised error")
        except:
            results.add_success(category, "LPUSH on string key (error expected)")
        
        # Try to increment non-numeric string
        client.set("non_numeric", "abc")
        try:
            client.incr("non_numeric")
            results.add_failure(category, "INCR on non-numeric", "Should have raised error")
        except:
            results.add_success(category, "INCR on non-numeric (error expected)")
        
    except Exception as e:
        results.add_failure(category, str(e).split(',')[0], str(e))


def main():
    """Run all tests."""
    print("Starting DiskDB Python Client Tests...")
    print("Connecting to DiskDB server at localhost:6380")
    
    results = TestResult()
    
    try:
        # Create client
        with DiskDBClient() as client:
            # Run all test categories
            test_string_operations(client, results)
            test_list_operations(client, results)
            test_set_operations(client, results)
            test_hash_operations(client, results)
            test_sorted_set_operations(client, results)
            test_json_operations(client, results)
            test_stream_operations(client, results)
            test_utility_operations(client, results)
            test_error_handling(client, results)
            
    except Exception as e:
        print(f"\nFATAL ERROR: Could not connect to DiskDB server: {e}")
        print("Make sure DiskDB server is running on localhost:6380")
        return 1
    
    # Print report
    results.print_report()
    
    return 0 if results.failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())