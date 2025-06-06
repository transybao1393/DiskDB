import socket
from typing import Optional


class DiskDBClient:
    """Python client for DiskDB server."""
    
    def __init__(self, host: str = 'localhost', port: int = 6380):
        """Initialize the DiskDB client.
        
        Args:
            host: The server host address
            port: The server port number
        """
        self.host = host
        self.port = port
        self.socket = None
        self.connect()
    
    def connect(self):
        """Establish connection to the DiskDB server."""
        if self.socket:
            self.close()
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.connect((self.host, self.port))
    
    def _send_command(self, command: str) -> str:
        """Send a command to the server and return the response.
        
        Args:
            command: The command string to send
            
        Returns:
            The server response
            
        Raises:
            ConnectionError: If not connected to server
        """
        if not self.socket:
            raise ConnectionError("Not connected to DiskDB server")
        
        self.socket.sendall((command + "\n").encode())
        response = self.socket.recv(1024).decode()
        return response.strip()
    
    def set(self, key: str, value: str) -> bool:
        """Store a key-value pair in the database.
        
        Args:
            key: The key to store
            value: The value to store
            
        Returns:
            True if successful, False otherwise
        """
        response = self._send_command(f"SET {key} {value}")
        return response == "OK"
    
    def get(self, key: str) -> Optional[str]:
        """Retrieve a value by key from the database.
        
        Args:
            key: The key to retrieve
            
        Returns:
            The value if found, None otherwise
        """
        response = self._send_command(f"GET {key}")
        if response.startswith("ERROR:"):
            return None
        return response
    
    def close(self):
        """Close the connection to the server."""
        if self.socket:
            self.socket.close()
            self.socket = None
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()


# Example usage
if __name__ == "__main__":
    # Using context manager
    with DiskDBClient() as client:
        # Set some values
        client.set("name", "John Doe")
        client.set("age", "30")
        client.set("city", "New York")
        
        # Get values
        print(f"Name: {client.get('name')}")
        print(f"Age: {client.get('age')}")
        print(f"City: {client.get('city')}")
        print(f"Unknown: {client.get('unknown')}")  # Should return None