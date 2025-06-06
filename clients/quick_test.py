#!/usr/bin/env python3
"""Quick test to verify basic connectivity."""

from diskdb_client import DiskDBClient

try:
    client = DiskDBClient()
    print("Connected to DiskDB server")
    
    # Test basic SET/GET
    result = client.set("test", "value")
    print(f"SET test value: {result}")
    
    value = client.get("test")
    print(f"GET test: {value}")
    
    client.close()
    print("Test completed successfully!")
    
except Exception as e:
    print(f"Error: {e}")
    import traceback
    traceback.print_exc()