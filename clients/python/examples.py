#!/usr/bin/env python3
"""
DiskDB Python Client Examples

This file demonstrates various use cases for DiskDB.
"""

from diskdb import DiskDB
import json
import time


def string_examples():
    """Demonstrate string operations."""
    print("\n=== String Operations ===")
    
    with DiskDB() as db:
        # Basic set/get
        db.set("name", "Alice")
        print(f"Name: {db.get('name')}")
        
        # Counter operations
        db.set("visits", "0")
        for i in range(5):
            count = db.incr("visits")
            print(f"Visit #{count}")
        
        # String building
        db.set("log", "Start")
        db.append("log", " -> Processing")
        db.append("log", " -> Complete")
        print(f"Log: {db.get('log')}")


def list_examples():
    """Demonstrate list operations as queue/stack."""
    print("\n=== List Operations (Queue/Stack) ===")
    
    with DiskDB() as db:
        db.delete("tasks")
        
        # Use as queue (FIFO)
        print("Queue example:")
        db.rpush("tasks", "task1", "task2", "task3")
        while db.llen("tasks") > 0:
            task = db.lpop("tasks")
            print(f"  Processing: {task}")
        
        # Use as stack (LIFO)
        print("\nStack example:")
        db.lpush("stack", "bottom", "middle", "top")
        while db.llen("stack") > 0:
            item = db.lpop("stack")
            print(f"  Popped: {item}")


def set_examples():
    """Demonstrate set operations."""
    print("\n=== Set Operations ===")
    
    with DiskDB() as db:
        db.delete("skills", "required_skills")
        
        # User skills
        db.sadd("skills", "python", "javascript", "sql", "docker")
        
        # Required skills for a job
        db.sadd("required_skills", "python", "sql", "kubernetes")
        
        print(f"User skills: {db.smembers('skills')}")
        print(f"Required skills: {db.smembers('required_skills')}")
        
        # Check individual skills
        for skill in ["python", "kubernetes", "java"]:
            has_skill = db.sismember("skills", skill)
            print(f"  Has {skill}: {has_skill}")


def hash_examples():
    """Demonstrate hash operations for object storage."""
    print("\n=== Hash Operations (Object Storage) ===")
    
    with DiskDB() as db:
        db.delete("user:1001")
        
        # Store user profile
        db.hset("user:1001", "username", "alice")
        db.hset("user:1001", "email", "alice@example.com")
        db.hset("user:1001", "created", "2024-01-15")
        db.hset("user:1001", "status", "active")
        
        # Get specific fields
        username = db.hget("user:1001", "username")
        email = db.hget("user:1001", "email")
        print(f"User: {username} ({email})")
        
        # Get all fields
        user_data = db.hgetall("user:1001")
        print("Full profile:")
        for field, value in user_data.items():
            print(f"  {field}: {value}")


def sorted_set_examples():
    """Demonstrate sorted sets for leaderboards."""
    print("\n=== Sorted Set Operations (Leaderboard) ===")
    
    with DiskDB() as db:
        db.delete("game:leaderboard")
        
        # Add player scores
        scores = {
            "alice": 2500,
            "bob": 1800,
            "charlie": 3200,
            "diana": 2900,
            "eve": 2100
        }
        db.zadd("game:leaderboard", scores)
        
        # Get top 3 players
        print("Top 3 players:")
        top_players = db.zrange("game:leaderboard", -3, -1, withscores=True)
        for rank, (player, score) in enumerate(reversed(top_players), 1):
            print(f"  #{rank} {player}: {score} points")
        
        # Get specific player's score
        player = "alice"
        score = db.zscore("game:leaderboard", player)
        print(f"\n{player}'s score: {score}")


def json_examples():
    """Demonstrate JSON operations."""
    print("\n=== JSON Operations ===")
    
    with DiskDB() as db:
        db.delete("config", "user:profile")
        
        # Store application config
        config = {
            "app": {
                "name": "MyApp",
                "version": "1.0.0",
                "settings": {
                    "debug": False,
                    "port": 8080,
                    "features": ["auth", "api", "websocket"]
                }
            }
        }
        db.json_set("config", "$", config)
        
        # Retrieve nested data
        app_name = db.json_get("config", "$.app.name")
        features = db.json_get("config", "$.app.settings.features")
        print(f"App: {app_name}")
        print(f"Features: {features}")
        
        # Store user profile with nested data
        profile = {
            "user": {
                "name": "John Doe",
                "age": 30,
                "preferences": {
                    "theme": "dark",
                    "language": "en",
                    "notifications": {
                        "email": True,
                        "push": False
                    }
                }
            }
        }
        db.json_set("user:profile", "$", profile)
        
        # Get specific preference
        theme = db.json_get("user:profile", "$.user.preferences.theme")
        print(f"\nUser theme preference: {theme}")


def stream_examples():
    """Demonstrate stream operations for event logging."""
    print("\n=== Stream Operations (Event Log) ===")
    
    with DiskDB() as db:
        db.delete("events:log")
        
        # Log some events
        events = [
            {"type": "user_login", "user": "alice", "ip": "192.168.1.1"},
            {"type": "api_call", "endpoint": "/api/data", "method": "GET"},
            {"type": "user_action", "user": "bob", "action": "create_post"},
            {"type": "system", "level": "info", "message": "Cache cleared"},
        ]
        
        print("Adding events to stream...")
        for event in events:
            event_id = db.xadd("events:log", event)
            print(f"  Added event {event_id}")
        
        # Read all events
        print("\nAll events in stream:")
        stream_entries = db.xrange("events:log", "-", "+")
        for entry in stream_entries:
            print(f"  ID: {entry['id']}")
            for field, value in entry['fields'].items():
                print(f"    {field}: {value}")
        
        print(f"\nTotal events: {db.xlen('events:log')}")


def utility_examples():
    """Demonstrate utility operations."""
    print("\n=== Utility Operations ===")
    
    with DiskDB() as db:
        # Set up different data types
        db.set("string_key", "value")
        db.lpush("list_key", "item")
        db.sadd("set_key", "member")
        db.hset("hash_key", "field", "value")
        
        # Check types
        print("Data types:")
        for key in ["string_key", "list_key", "set_key", "hash_key", "nonexistent"]:
            key_type = db.type(key)
            print(f"  {key}: {key_type}")
        
        # Check existence
        keys_to_check = ["string_key", "list_key", "nonexistent"]
        exists_count = db.exists(*keys_to_check)
        print(f"\nExisting keys: {exists_count} out of {len(keys_to_check)}")
        
        # Clean up
        deleted = db.delete("string_key", "list_key", "set_key", "hash_key")
        print(f"Deleted {deleted} keys")


def real_world_example():
    """A real-world example: Simple task management system."""
    print("\n=== Real World Example: Task Management ===")
    
    with DiskDB() as db:
        # Clean up
        db.delete("tasks:pending", "tasks:completed", "task:*")
        
        # Add tasks to pending queue
        tasks = [
            "Send newsletter",
            "Update documentation", 
            "Review pull requests",
            "Deploy to production"
        ]
        
        print("Adding tasks...")
        for task in tasks:
            db.lpush("tasks:pending", task)
            print(f"  Added: {task}")
        
        # Process tasks
        print("\nProcessing tasks...")
        while db.llen("tasks:pending") > 0:
            # Get next task
            task = db.rpop("tasks:pending")
            print(f"  Working on: {task}")
            
            # Simulate work
            time.sleep(0.5)
            
            # Mark as completed
            db.lpush("tasks:completed", task)
            print(f"  Completed: {task}")
        
        # Show summary
        completed_count = db.llen("tasks:completed")
        print(f"\nAll done! Completed {completed_count} tasks")
        
        # Show completed tasks
        print("\nCompleted tasks:")
        completed = db.lrange("tasks:completed", 0, -1)
        for task in completed:
            print(f"  ✓ {task}")


if __name__ == "__main__":
    print("DiskDB Python Client Examples")
    print("=" * 50)
    
    try:
        # Run all examples
        string_examples()
        list_examples()
        set_examples()
        hash_examples()
        sorted_set_examples()
        json_examples()
        stream_examples()
        utility_examples()
        real_world_example()
        
        print("\n✅ All examples completed successfully!")
        
    except Exception as e:
        print(f"\n❌ Error: {e}")
        print("Make sure DiskDB server is running on localhost:6380")