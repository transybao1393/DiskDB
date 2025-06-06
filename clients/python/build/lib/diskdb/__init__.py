"""
DiskDB Python Client Library

A comprehensive Python client for DiskDB, supporting all Redis-compatible data types
and operations including Strings, Lists, Sets, Hashes, Sorted Sets, JSON, and Streams.

Example:
    from diskdb import DiskDB
    
    # Connect to DiskDB
    db = DiskDB()
    
    # Use it
    db.set("key", "value")
    value = db.get("key")
"""

from .client import DiskDB
from .exceptions import (
    DiskDBError,
    ConnectionError,
    CommandError,
    TypeMismatchError
)

__version__ = "0.1.0"
__author__ = "DiskDB Team"
__all__ = [
    "DiskDB",
    "DiskDBError",
    "ConnectionError", 
    "CommandError",
    "TypeMismatchError"
]