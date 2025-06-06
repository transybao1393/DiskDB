package main

import (
	"fmt"
	"log"
)

// Copy of the client code for testing
import (
	"bufio"
	"net"
	"strings"
)

type Client struct {
	conn   net.Conn
	reader *bufio.Reader
}

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

func (c *Client) Close() error {
	if c.conn != nil {
		return c.conn.Close()
	}
	return nil
}

func main() {
	fmt.Println("Testing DiskDB Go client...")
	
	client, err := NewClient("localhost:6380")
	if err != nil {
		log.Fatal("Failed to connect:", err)
	}
	defer client.Close()
	
	// Test SET operations
	fmt.Println("Setting test values...")
	if err := client.Set("language", "Go"); err != nil {
		log.Fatal("Failed to set language:", err)
	}
	if err := client.Set("version", "1.21"); err != nil {
		log.Fatal("Failed to set version:", err)
	}
	
	// Test GET operations
	fmt.Println("Getting test values...")
	
	language, err := client.Get("language")
	if err != nil {
		log.Fatal("Failed to get language:", err)
	}
	fmt.Printf("Language: %s\n", language)
	
	version, err := client.Get("version")
	if err != nil {
		log.Fatal("Failed to get version:", err)
	}
	fmt.Printf("Version: %s\n", version)
	
	// Test non-existent key
	_, err = client.Get("nonexistent")
	if err != nil {
		fmt.Printf("Expected error for non-existent key: %v\n", err)
	}
	
	fmt.Println("All tests passed!")
}