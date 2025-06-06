"""
DiskDB Python Client

A comprehensive client library for DiskDB with support for all data types.
"""

import socket
import json
from typing import Optional, List, Dict, Any, Tuple, Union, Set as TypeSet
from contextlib import contextmanager

from .exceptions import ConnectionError, CommandError, TypeMismatchError, TimeoutError


class DiskDB:
    """
    DiskDB client for Python.
    
    Supports all Redis-compatible operations for:
    - Strings
    - Lists
    - Sets
    - Hashes
    - Sorted Sets
    - JSON
    - Streams
    
    Example:
        db = DiskDB()
        db.set("key", "value")
        value = db.get("key")
    """
    
    def __init__(self, host: str = 'localhost', port: int = 6380, timeout: float = 5.0):
        """
        Initialize DiskDB client.
        
        Args:
            host: Server hostname (default: localhost)
            port: Server port (default: 6380)
            timeout: Socket timeout in seconds (default: 5.0)
        """
        self.host = host
        self.port = port
        self.timeout = timeout
        self.socket = None
        self.buffer = b""
        self.connect()
    
    def connect(self):
        """Connect to DiskDB server."""
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.settimeout(self.timeout)
            self.socket.connect((self.host, self.port))
            self.buffer = b""
        except socket.error as e:
            raise ConnectionError(f"Failed to connect to {self.host}:{self.port}: {e}")
    
    def close(self):
        """Close connection to server."""
        if self.socket:
            try:
                self.socket.close()
            except:
                pass
            self.socket = None
    
    def _ensure_connected(self):
        """Ensure connection is active."""
        if not self.socket:
            self.connect()
    
    def _read_line(self) -> str:
        """Read a single line from the socket."""
        while b"\n" not in self.buffer:
            try:
                chunk = self.socket.recv(1024)
                if not chunk:
                    raise ConnectionError("Connection closed by server")
                self.buffer += chunk
            except socket.timeout:
                raise TimeoutError("Operation timed out")
        
        line, self.buffer = self.buffer.split(b"\n", 1)
        return line.decode().strip()
    
    def _send_command(self, command: str) -> str:
        """Send command and receive single-line response."""
        self._ensure_connected()
        
        try:
            self.socket.send(f"{command}\n".encode())
            response = self._read_line()
            
            if response.startswith("ERROR:"):
                error_msg = response[6:].strip()
                if "WRONGTYPE" in error_msg:
                    raise TypeMismatchError(error_msg)
                raise CommandError(error_msg)
            
            return response
        except socket.error as e:
            self.close()
            raise ConnectionError(f"Connection error: {e}")
    
    def _read_array(self, count: Optional[int] = None) -> List[str]:
        """Read multiple lines as array response."""
        result = []
        try:
            # Use shorter timeout for array reading
            old_timeout = self.socket.gettimeout()
            self.socket.settimeout(0.5)
            
            if count:
                # Read exact number of lines
                for _ in range(count):
                    result.append(self._read_line())
            else:
                # Read until timeout
                while True:
                    line = self._read_line()
                    if not line:
                        break
                    result.append(line)
                    
        except (socket.timeout, TimeoutError):
            # Expected when no more data
            pass
        finally:
            # Restore original timeout
            self.socket.settimeout(self.timeout)
        
        return result
    
    # String Operations
    
    def set(self, key: str, value: str) -> bool:
        """
        Set key to hold the string value.
        
        Args:
            key: The key
            value: The value to set
            
        Returns:
            True if successful
        """
        response = self._send_command(f"SET {key} {value}")
        return response == "OK"
    
    def get(self, key: str) -> Optional[str]:
        """
        Get the value of key.
        
        Args:
            key: The key
            
        Returns:
            The value or None if key doesn't exist
        """
        response = self._send_command(f"GET {key}")
        return None if response == "(nil)" else response
    
    def incr(self, key: str) -> int:
        """
        Increment the integer value of key by 1.
        
        Args:
            key: The key
            
        Returns:
            The value after increment
        """
        response = self._send_command(f"INCR {key}")
        return int(response)
    
    def decr(self, key: str) -> int:
        """
        Decrement the integer value of key by 1.
        
        Args:
            key: The key
            
        Returns:
            The value after decrement
        """
        response = self._send_command(f"DECR {key}")
        return int(response)
    
    def incrby(self, key: str, increment: int) -> int:
        """
        Increment the integer value of key by the given amount.
        
        Args:
            key: The key
            increment: Amount to increment by
            
        Returns:
            The value after increment
        """
        response = self._send_command(f"INCRBY {key} {increment}")
        return int(response)
    
    def append(self, key: str, value: str) -> int:
        """
        Append a value to a key.
        
        Args:
            key: The key
            value: The value to append
            
        Returns:
            The length of the string after append
        """
        response = self._send_command(f"APPEND {key} {value}")
        return int(response)
    
    # List Operations
    
    def lpush(self, key: str, *values: str) -> int:
        """
        Insert values at the head of the list.
        
        Args:
            key: The key
            *values: Values to push
            
        Returns:
            The length of the list after push
        """
        values_str = " ".join(values)
        response = self._send_command(f"LPUSH {key} {values_str}")
        return int(response)
    
    def rpush(self, key: str, *values: str) -> int:
        """
        Insert values at the tail of the list.
        
        Args:
            key: The key
            *values: Values to push
            
        Returns:
            The length of the list after push
        """
        values_str = " ".join(values)
        response = self._send_command(f"RPUSH {key} {values_str}")
        return int(response)
    
    def lpop(self, key: str) -> Optional[str]:
        """
        Remove and return the first element of the list.
        
        Args:
            key: The key
            
        Returns:
            The popped element or None if list is empty
        """
        response = self._send_command(f"LPOP {key}")
        return None if response == "(nil)" else response
    
    def rpop(self, key: str) -> Optional[str]:
        """
        Remove and return the last element of the list.
        
        Args:
            key: The key
            
        Returns:
            The popped element or None if list is empty
        """
        response = self._send_command(f"RPOP {key}")
        return None if response == "(nil)" else response
    
    def lrange(self, key: str, start: int, stop: int) -> List[str]:
        """
        Return the specified elements of the list.
        
        Args:
            key: The key
            start: Start index (0-based, can be negative)
            stop: Stop index (inclusive, can be negative)
            
        Returns:
            List of elements
        """
        self._ensure_connected()
        self.socket.send(f"LRANGE {key} {start} {stop}\n".encode())
        
        # Read the array response
        return self._read_array()
    
    def llen(self, key: str) -> int:
        """
        Return the length of the list.
        
        Args:
            key: The key
            
        Returns:
            The length of the list
        """
        response = self._send_command(f"LLEN {key}")
        return int(response)
    
    # Set Operations
    
    def sadd(self, key: str, *members: str) -> int:
        """
        Add members to a set.
        
        Args:
            key: The key
            *members: Members to add
            
        Returns:
            The number of members added
        """
        members_str = " ".join(members)
        response = self._send_command(f"SADD {key} {members_str}")
        return int(response)
    
    def srem(self, key: str, *members: str) -> int:
        """
        Remove members from a set.
        
        Args:
            key: The key
            *members: Members to remove
            
        Returns:
            The number of members removed
        """
        members_str = " ".join(members)
        response = self._send_command(f"SREM {key} {members_str}")
        return int(response)
    
    def sismember(self, key: str, member: str) -> bool:
        """
        Check if member is in set.
        
        Args:
            key: The key
            member: The member to check
            
        Returns:
            True if member exists in set
        """
        response = self._send_command(f"SISMEMBER {key} {member}")
        return response == "1"
    
    def smembers(self, key: str) -> TypeSet[str]:
        """
        Return all members of the set.
        
        Args:
            key: The key
            
        Returns:
            Set of all members
        """
        self._ensure_connected()
        self.socket.send(f"SMEMBERS {key}\n".encode())
        members = self._read_array()
        return set(members)
    
    def scard(self, key: str) -> int:
        """
        Return the number of members in set.
        
        Args:
            key: The key
            
        Returns:
            The cardinality (number of members) of the set
        """
        response = self._send_command(f"SCARD {key}")
        return int(response)
    
    # Hash Operations
    
    def hset(self, key: str, field: str, value: str) -> int:
        """
        Set field in hash to value.
        
        Args:
            key: The key
            field: The field name
            value: The value
            
        Returns:
            1 if field is new, 0 if field existed
        """
        response = self._send_command(f"HSET {key} {field} {value}")
        return int(response)
    
    def hget(self, key: str, field: str) -> Optional[str]:
        """
        Get the value of field in hash.
        
        Args:
            key: The key
            field: The field name
            
        Returns:
            The value or None if field doesn't exist
        """
        response = self._send_command(f"HGET {key} {field}")
        return None if response == "(nil)" else response
    
    def hdel(self, key: str, *fields: str) -> int:
        """
        Delete fields from hash.
        
        Args:
            key: The key
            *fields: Fields to delete
            
        Returns:
            The number of fields removed
        """
        fields_str = " ".join(fields)
        response = self._send_command(f"HDEL {key} {fields_str}")
        return int(response)
    
    def hgetall(self, key: str) -> Dict[str, str]:
        """
        Get all fields and values in hash.
        
        Args:
            key: The key
            
        Returns:
            Dictionary of field-value pairs
        """
        self._ensure_connected()
        self.socket.send(f"HGETALL {key}\n".encode())
        
        result = {}
        lines = self._read_array()
        
        # Parse field-value pairs
        for i in range(0, len(lines), 2):
            if i + 1 < len(lines):
                result[lines[i]] = lines[i + 1]
        
        return result
    
    def hexists(self, key: str, field: str) -> bool:
        """
        Check if field exists in hash.
        
        Args:
            key: The key
            field: The field name
            
        Returns:
            True if field exists
        """
        response = self._send_command(f"HEXISTS {key} {field}")
        return response == "1"
    
    # Sorted Set Operations
    
    def zadd(self, key: str, mapping: Dict[str, float]) -> int:
        """
        Add members to sorted set with scores.
        
        Args:
            key: The key
            mapping: Dictionary of member:score pairs
            
        Returns:
            The number of members added
        """
        parts = []
        for member, score in mapping.items():
            parts.extend([str(score), member])
        args_str = " ".join(parts)
        response = self._send_command(f"ZADD {key} {args_str}")
        return int(response)
    
    def zrem(self, key: str, *members: str) -> int:
        """
        Remove members from sorted set.
        
        Args:
            key: The key
            *members: Members to remove
            
        Returns:
            The number of members removed
        """
        members_str = " ".join(members)
        response = self._send_command(f"ZREM {key} {members_str}")
        return int(response)
    
    def zscore(self, key: str, member: str) -> Optional[float]:
        """
        Get the score of member in sorted set.
        
        Args:
            key: The key
            member: The member
            
        Returns:
            The score or None if member doesn't exist
        """
        response = self._send_command(f"ZSCORE {key} {member}")
        return None if response == "(nil)" else float(response)
    
    def zrange(self, key: str, start: int, stop: int, withscores: bool = False) -> Union[List[str], List[Tuple[str, float]]]:
        """
        Return range of members in sorted set.
        
        Args:
            key: The key
            start: Start rank (0-based, can be negative)
            stop: Stop rank (inclusive, can be negative)
            withscores: Include scores in result
            
        Returns:
            List of members or list of (member, score) tuples
        """
        command = f"ZRANGE {key} {start} {stop}"
        if withscores:
            command += " WITHSCORES"
        
        self._ensure_connected()
        self.socket.send(f"{command}\n".encode())
        
        lines = self._read_array()
        
        if withscores and len(lines) > 1:
            # Convert to list of tuples
            result = []
            for i in range(0, len(lines), 2):
                if i + 1 < len(lines):
                    result.append((lines[i], float(lines[i + 1])))
            return result
        
        return lines
    
    def zcard(self, key: str) -> int:
        """
        Return the number of members in sorted set.
        
        Args:
            key: The key
            
        Returns:
            The cardinality of the sorted set
        """
        response = self._send_command(f"ZCARD {key}")
        return int(response)
    
    # JSON Operations
    
    def json_set(self, key: str, path: str, value: Any) -> bool:
        """
        Set JSON value at path.
        
        Args:
            key: The key
            path: JSON path (use "$" for root)
            value: Python object to store as JSON
            
        Returns:
            True if successful
        """
        json_str = json.dumps(value, separators=(',', ':'))
        response = self._send_command(f"JSON.SET {key} {path} {json_str}")
        return response == "OK"
    
    def json_get(self, key: str, path: str) -> Any:
        """
        Get JSON value at path.
        
        Args:
            key: The key
            path: JSON path (use "$" for root)
            
        Returns:
            Python object parsed from JSON
        """
        response = self._send_command(f"JSON.GET {key} {path}")
        if response == "(nil)":
            return None
        return json.loads(response)
    
    def json_del(self, key: str, path: str) -> int:
        """
        Delete JSON value at path.
        
        Args:
            key: The key
            path: JSON path (use "$" for root)
            
        Returns:
            Number of paths deleted
        """
        response = self._send_command(f"JSON.DEL {key} {path}")
        return int(response)
    
    # Stream Operations
    
    def xadd(self, key: str, fields: Dict[str, str], id: str = "*") -> str:
        """
        Add entry to stream.
        
        Args:
            key: The key
            fields: Dictionary of field-value pairs
            id: Entry ID (use "*" for auto-generated)
            
        Returns:
            The entry ID
        """
        parts = [id]
        for field, value in fields.items():
            parts.extend([field, value])
        args_str = " ".join(parts)
        response = self._send_command(f"XADD {key} {args_str}")
        return response
    
    def xlen(self, key: str) -> int:
        """
        Return the number of entries in stream.
        
        Args:
            key: The key
            
        Returns:
            The number of entries
        """
        response = self._send_command(f"XLEN {key}")
        return int(response)
    
    def xrange(self, key: str, start: str = "-", end: str = "+", count: Optional[int] = None) -> List[Dict[str, Any]]:
        """
        Return range of entries from stream.
        
        Args:
            key: The key
            start: Start ID (use "-" for minimum)
            end: End ID (use "+" for maximum)
            count: Maximum number of entries to return
            
        Returns:
            List of entries with id and fields
        """
        command = f"XRANGE {key} {start} {end}"
        if count:
            command += f" COUNT {count}"
        
        self._ensure_connected()
        self.socket.send(f"{command}\n".encode())
        
        # Simple parsing for stream entries
        entries = []
        lines = self._read_array()
        
        i = 0
        while i < len(lines):
            if "-" in lines[i]:  # Entry ID
                entry_id = lines[i]
                fields = {}
                i += 1
                
                # Read field-value pairs
                while i < len(lines) and "-" not in lines[i]:
                    if i + 1 < len(lines):
                        fields[lines[i]] = lines[i + 1]
                        i += 2
                    else:
                        break
                
                entries.append({"id": entry_id, "fields": fields})
            else:
                i += 1
        
        return entries
    
    # Utility Operations
    
    def type(self, key: str) -> str:
        """
        Return the type of key.
        
        Args:
            key: The key
            
        Returns:
            Type string: string, list, set, zset, hash, json, stream, or none
        """
        response = self._send_command(f"TYPE {key}")
        return response
    
    def exists(self, *keys: str) -> int:
        """
        Check if keys exist.
        
        Args:
            *keys: Keys to check
            
        Returns:
            Number of keys that exist
        """
        keys_str = " ".join(keys)
        response = self._send_command(f"EXISTS {keys_str}")
        return int(response)
    
    def delete(self, *keys: str) -> int:
        """
        Delete keys.
        
        Args:
            *keys: Keys to delete
            
        Returns:
            Number of keys deleted
        """
        keys_str = " ".join(keys)
        response = self._send_command(f"DEL {keys_str}")
        return int(response)
    
    # Aliases for common operations
    setex = set  # For compatibility
    setnx = set  # For compatibility
    
    # Context manager support
    
    def __enter__(self):
        """Enter context manager."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit context manager."""
        self.close()
    
    # Pipeline support (simple batching)
    
    @contextmanager
    def pipeline(self):
        """
        Create a pipeline for batching commands.
        
        Note: This is a simple implementation that doesn't provide
        true pipelining but allows for cleaner code organization.
        
        Example:
            with db.pipeline() as pipe:
                pipe.set("key1", "value1")
                pipe.set("key2", "value2")
                pipe.incr("counter")
        """
        yield self