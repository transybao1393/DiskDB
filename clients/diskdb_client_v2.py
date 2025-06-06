#!/usr/bin/env python3
"""
DiskDB Python Client V2 - Improved version with better array handling

A Python client for interacting with DiskDB server.
Supports all Redis-like data types: Strings, Lists, Sets, Hashes, Sorted Sets, JSON, and Streams.
"""

import socket
import json
from typing import Optional, List, Dict, Any, Tuple, Union


class DiskDBClient:
    """Python client for DiskDB server."""
    
    def __init__(self, host: str = 'localhost', port: int = 6380):
        """
        Initialize DiskDB client.
        
        Args:
            host: Server hostname (default: localhost)
            port: Server port (default: 6380)
        """
        self.host = host
        self.port = port
        self.socket = None
        self.buffer = b""
        self.connect()
    
    def connect(self):
        """Connect to DiskDB server."""
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.settimeout(5.0)  # 5 second timeout
        self.socket.connect((self.host, self.port))
        self.buffer = b""
    
    def close(self):
        """Close connection to server."""
        if self.socket:
            self.socket.close()
    
    def _read_line(self) -> str:
        """Read a single line from the socket."""
        while b"\n" not in self.buffer:
            chunk = self.socket.recv(1024)
            if not chunk:
                raise ConnectionError("Connection closed")
            self.buffer += chunk
        
        line, self.buffer = self.buffer.split(b"\n", 1)
        return line.decode().strip()
    
    def _send_command(self, command: str) -> str:
        """
        Send command to server and receive response.
        
        Args:
            command: Command string to send
            
        Returns:
            Server response as string
        """
        if not self.socket:
            self.connect()
        
        # Send command
        self.socket.send(f"{command}\n".encode())
        
        # Read response
        return self._read_line()
    
    # String operations
    def set(self, key: str, value: str) -> bool:
        """Set a key-value pair."""
        response = self._send_command(f"SET {key} {value}")
        return response == "OK"
    
    def get(self, key: str) -> Optional[str]:
        """Get value for a key."""
        response = self._send_command(f"GET {key}")
        return None if response == "(nil)" else response
    
    def incr(self, key: str) -> int:
        """Increment a key's value."""
        response = self._send_command(f"INCR {key}")
        return int(response)
    
    def decr(self, key: str) -> int:
        """Decrement a key's value."""
        response = self._send_command(f"DECR {key}")
        return int(response)
    
    def incrby(self, key: str, delta: int) -> int:
        """Increment a key's value by delta."""
        response = self._send_command(f"INCRBY {key} {delta}")
        return int(response)
    
    def append(self, key: str, value: str) -> int:
        """Append to a string value."""
        response = self._send_command(f"APPEND {key} {value}")
        return int(response)
    
    # List operations
    def lpush(self, key: str, *values: str) -> int:
        """Push values to the left of a list."""
        values_str = " ".join(values)
        response = self._send_command(f"LPUSH {key} {values_str}")
        return int(response)
    
    def rpush(self, key: str, *values: str) -> int:
        """Push values to the right of a list."""
        values_str = " ".join(values)
        response = self._send_command(f"RPUSH {key} {values_str}")
        return int(response)
    
    def lpop(self, key: str) -> Optional[str]:
        """Pop from the left of a list."""
        response = self._send_command(f"LPOP {key}")
        return None if response == "(nil)" else response
    
    def rpop(self, key: str) -> Optional[str]:
        """Pop from the right of a list."""
        response = self._send_command(f"RPOP {key}")
        return None if response == "(nil)" else response
    
    def lrange(self, key: str, start: int, stop: int) -> List[str]:
        """Get range of elements from a list."""
        # Send command
        self.socket.send(f"LRANGE {key} {start} {stop}\n".encode())
        
        # Read response lines until we get all elements
        result = []
        # We'll read lines until we encounter an empty line or timeout
        try:
            while True:
                line = self._read_line()
                if not line:
                    break
                result.append(line)
        except socket.timeout:
            # We've read all available lines
            pass
        
        return result
    
    def llen(self, key: str) -> int:
        """Get length of a list."""
        response = self._send_command(f"LLEN {key}")
        return int(response)
    
    # Set operations
    def sadd(self, key: str, *members: str) -> int:
        """Add members to a set."""
        members_str = " ".join(members)
        response = self._send_command(f"SADD {key} {members_str}")
        return int(response)
    
    def srem(self, key: str, *members: str) -> int:
        """Remove members from a set."""
        members_str = " ".join(members)
        response = self._send_command(f"SREM {key} {members_str}")
        return int(response)
    
    def sismember(self, key: str, member: str) -> bool:
        """Check if member exists in set."""
        response = self._send_command(f"SISMEMBER {key} {member}")
        return response == "1"
    
    def smembers(self, key: str) -> List[str]:
        """Get all members of a set."""
        # Send command
        self.socket.send(f"SMEMBERS {key}\n".encode())
        
        # Read response lines
        result = []
        try:
            # Set a shorter timeout for reading members
            self.socket.settimeout(0.5)
            while True:
                line = self._read_line()
                if not line:
                    break
                result.append(line)
        except socket.timeout:
            pass
        finally:
            # Restore normal timeout
            self.socket.settimeout(5.0)
        
        return result
    
    def scard(self, key: str) -> int:
        """Get cardinality of a set."""
        response = self._send_command(f"SCARD {key}")
        return int(response)
    
    # Hash operations
    def hset(self, key: str, field: str, value: str) -> int:
        """Set field in hash."""
        response = self._send_command(f"HSET {key} {field} {value}")
        return int(response)
    
    def hget(self, key: str, field: str) -> Optional[str]:
        """Get field from hash."""
        response = self._send_command(f"HGET {key} {field}")
        return None if response == "(nil)" else response
    
    def hdel(self, key: str, *fields: str) -> int:
        """Delete fields from hash."""
        fields_str = " ".join(fields)
        response = self._send_command(f"HDEL {key} {fields_str}")
        return int(response)
    
    def hgetall(self, key: str) -> Dict[str, str]:
        """Get all fields and values from hash."""
        # Send command
        self.socket.send(f"HGETALL {key}\n".encode())
        
        # Read field-value pairs
        result = {}
        try:
            self.socket.settimeout(0.5)
            while True:
                field = self._read_line()
                if not field:
                    break
                value = self._read_line()
                result[field] = value
        except socket.timeout:
            pass
        finally:
            self.socket.settimeout(5.0)
        
        return result
    
    def hexists(self, key: str, field: str) -> bool:
        """Check if field exists in hash."""
        response = self._send_command(f"HEXISTS {key} {field}")
        return response == "1"
    
    # Sorted Set operations
    def zadd(self, key: str, mapping: Dict[str, float]) -> int:
        """Add members to sorted set with scores."""
        parts = []
        for member, score in mapping.items():
            parts.extend([str(score), member])
        args_str = " ".join(parts)
        response = self._send_command(f"ZADD {key} {args_str}")
        return int(response)
    
    def zrem(self, key: str, *members: str) -> int:
        """Remove members from sorted set."""
        members_str = " ".join(members)
        response = self._send_command(f"ZREM {key} {members_str}")
        return int(response)
    
    def zscore(self, key: str, member: str) -> Optional[float]:
        """Get score of member in sorted set."""
        response = self._send_command(f"ZSCORE {key} {member}")
        return None if response == "(nil)" else float(response)
    
    def zrange(self, key: str, start: int, stop: int, withscores: bool = False) -> Union[List[str], List[Tuple[str, float]]]:
        """Get range of members from sorted set."""
        command = f"ZRANGE {key} {start} {stop}"
        if withscores:
            command += " WITHSCORES"
        
        # Send command
        self.socket.send(f"{command}\n".encode())
        
        # Read response
        result = []
        try:
            self.socket.settimeout(0.5)
            while True:
                line = self._read_line()
                if not line:
                    break
                result.append(line)
        except socket.timeout:
            pass
        finally:
            self.socket.settimeout(5.0)
        
        if withscores and len(result) > 1:
            # Convert to list of tuples
            tuples = []
            for i in range(0, len(result), 2):
                if i + 1 < len(result):
                    tuples.append((result[i], float(result[i + 1])))
            return tuples
        
        return result
    
    def zcard(self, key: str) -> int:
        """Get cardinality of sorted set."""
        response = self._send_command(f"ZCARD {key}")
        return int(response)
    
    # JSON operations
    def json_set(self, key: str, path: str, value: Any) -> bool:
        """Set JSON value at path."""
        json_str = json.dumps(value, separators=(',', ':'))
        response = self._send_command(f"JSON.SET {key} {path} {json_str}")
        return response == "OK"
    
    def json_get(self, key: str, path: str) -> Any:
        """Get JSON value at path."""
        response = self._send_command(f"JSON.GET {key} {path}")
        if response == "(nil)":
            return None
        return json.loads(response)
    
    def json_del(self, key: str, path: str) -> int:
        """Delete JSON value at path."""
        response = self._send_command(f"JSON.DEL {key} {path}")
        return int(response)
    
    # Stream operations
    def xadd(self, key: str, fields: Dict[str, str], id: str = "*") -> str:
        """Add entry to stream."""
        parts = [id]
        for field, value in fields.items():
            parts.extend([field, value])
        args_str = " ".join(parts)
        response = self._send_command(f"XADD {key} {args_str}")
        return response
    
    def xlen(self, key: str) -> int:
        """Get length of stream."""
        response = self._send_command(f"XLEN {key}")
        return int(response)
    
    def xrange(self, key: str, start: str = "-", end: str = "+", count: Optional[int] = None) -> List[Dict[str, Any]]:
        """Get range of entries from stream."""
        command = f"XRANGE {key} {start} {end}"
        if count:
            command += f" COUNT {count}"
        
        # Send command
        self.socket.send(f"{command}\n".encode())
        
        # Read stream entries
        entries = []
        try:
            self.socket.settimeout(0.5)
            while True:
                entry_id = self._read_line()
                if not entry_id:
                    break
                
                # Read fields for this entry
                fields = {}
                while True:
                    try:
                        field_name = self._read_line()
                        if not field_name or field_name.startswith("-"):
                            # Next entry ID
                            entry_id = field_name
                            break
                        field_value = self._read_line()
                        fields[field_name] = field_value
                    except socket.timeout:
                        break
                
                if fields:
                    entries.append({"id": entry_id, "fields": fields})
                    
                if not entry_id or not entry_id.startswith("-"):
                    break
                    
        except socket.timeout:
            pass
        finally:
            self.socket.settimeout(5.0)
        
        return entries
    
    # Utility operations
    def type(self, key: str) -> str:
        """Get type of key."""
        return self._send_command(f"TYPE {key}")
    
    def exists(self, *keys: str) -> int:
        """Check if keys exist."""
        keys_str = " ".join(keys)
        response = self._send_command(f"EXISTS {keys_str}")
        return int(response)
    
    def delete(self, *keys: str) -> int:
        """Delete keys."""
        keys_str = " ".join(keys)
        response = self._send_command(f"DEL {keys_str}")
        return int(response)
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()