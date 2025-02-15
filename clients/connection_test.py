import socket

def send_command(command):
    """Send a command to the database and return the response."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect(("0.0.0.0", 6380))  # Change port if necessary
        s.sendall(command.encode() + b"\n")
        response = s.recv(1024).decode()
        return response.strip()

# Test the database connection
if __name__ == "__main__":
    print("Testing DiskDB connection...")

    # SET a key-value pair
    print("Sending: SET mykey myvalue")
    print("Response:", send_command("SET mykey myvalue"))

    # GET the value
    print("Sending: GET mykey")
    print("Response:", send_command("GET mykey"))

    # Try getting a non-existent key
    print("Sending: GET unknown_key")
    print("Response:", send_command("GET unknown_key"))

    # Send an invalid command
    print("Sending: INVALID_CMD")
    print("Response:", send_command("INVALID_CMD"))