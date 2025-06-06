package diskdb

import (
	"bufio"
	"fmt"
	"net"
	"strings"
)

// Client represents a DiskDB client connection
type Client struct {
	host   string
	port   int
	conn   net.Conn
	reader *bufio.Reader
}

// NewClient creates a new DiskDB client
func NewClient(address string) (*Client, error) {
	conn, err := net.Dial("tcp", address)
	if err != nil {
		return nil, err
	}
	
	return &Client{
		conn:   conn,
		reader: bufio.NewReader(conn),
	}, nil
}

// sendCommand sends a command to the server and returns the response
func (c *Client) sendCommand(command string) (string, error) {
	_, err := c.conn.Write([]byte(command + "\n"))
	if err != nil {
		return "", err
	}
	
	response, err := c.reader.ReadString('\n')
	if err != nil {
		return "", err
	}
	
	return strings.TrimSpace(response), nil
}

// Set stores a key-value pair in the database
func (c *Client) Set(key, value string) error {
	response, err := c.sendCommand(fmt.Sprintf("SET %s %s", key, value))
	if err != nil {
		return err
	}
	
	if response != "OK" {
		return fmt.Errorf("set failed: %s", response)
	}
	
	return nil
}

// Get retrieves a value by key from the database
func (c *Client) Get(key string) (string, error) {
	response, err := c.sendCommand(fmt.Sprintf("GET %s", key))
	if err != nil {
		return "", err
	}
	
	if strings.HasPrefix(response, "ERROR:") {
		return "", fmt.Errorf("key not found: %s", key)
	}
	
	return response, nil
}

// Close closes the connection to the server
func (c *Client) Close() error {
	if c.conn != nil {
		return c.conn.Close()
	}
	return nil
}

// Example usage (can be moved to a separate file)
func ExampleUsage() {
	client, err := NewClient("localhost:6380")
	if err != nil {
		panic(err)
	}
	defer client.Close()
	
	// Set some values
	err = client.Set("name", "Jane Doe")
	if err != nil {
		panic(err)
	}
	
	// Get values
	value, err := client.Get("name")
	if err != nil {
		panic(err)
	}
	fmt.Printf("Name: %s\n", value)
}