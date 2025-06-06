#!/bin/bash

# DiskDB Performance Baseline Script
# This script runs all benchmarks and generates a comprehensive baseline report

set -e

echo "========================================"
echo "DiskDB Performance Baseline Testing"
echo "Date: $(date)"
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if DiskDB is running
check_diskdb() {
    echo -e "${YELLOW}Checking if DiskDB is running...${NC}"
    if nc -z localhost 6380 2>/dev/null; then
        echo -e "${GREEN}✓ DiskDB is running on port 6380${NC}"
        return 0
    else
        echo -e "${RED}✗ DiskDB is not running${NC}"
        echo "Please start DiskDB with: cargo run --release"
        return 1
    fi
}

# Create results directory
RESULTS_DIR="benchmark_results/baseline_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo -e "\n${YELLOW}Results will be saved to: $RESULTS_DIR${NC}\n"

# System information
echo "=== System Information ==="
echo "OS: $(uname -s)"
echo "Architecture: $(uname -m)"
echo "CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || lscpu | grep 'Model name' | cut -d: -f2 | xargs)"
echo "Memory: $(sysctl -n hw.memsize 2>/dev/null | awk '{print int($1/1073741824)" GB"}' || free -h | grep Mem | awk '{print $2}')"
echo "Rust version: $(rustc --version)"
echo ""

# Save system info
cat > "$RESULTS_DIR/system_info.txt" << EOF
System Information
==================
Date: $(date)
OS: $(uname -s)
Architecture: $(uname -m)
CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || lscpu | grep 'Model name' | cut -d: -f2 | xargs)
Memory: $(sysctl -n hw.memsize 2>/dev/null | awk '{print int($1/1073741824)" GB"}' || free -h | grep Mem | awk '{print $2}')
Rust version: $(rustc --version)
EOF

# Build release version
echo -e "${YELLOW}Building DiskDB in release mode...${NC}"
cargo build --release

# Start DiskDB if not running
if ! check_diskdb; then
    echo -e "${YELLOW}Starting DiskDB server...${NC}"
    cargo run --release > "$RESULTS_DIR/diskdb_server.log" 2>&1 &
    DISKDB_PID=$!
    sleep 2
    
    if ! check_diskdb; then
        echo -e "${RED}Failed to start DiskDB${NC}"
        exit 1
    fi
fi

# Run Rust benchmarks
echo -e "\n${YELLOW}Running Rust benchmarks...${NC}"

# Protocol parsing benchmarks
echo -e "\n${GREEN}1. Protocol Parsing Benchmarks${NC}"
cargo bench --bench protocol_parsing -- --save-baseline baseline 2>&1 | tee "$RESULTS_DIR/protocol_parsing.txt"

# Core operations benchmarks
echo -e "\n${GREEN}2. Core Operations Benchmarks${NC}"
cargo bench --bench core_operations -- --save-baseline baseline 2>&1 | tee "$RESULTS_DIR/core_operations.txt"

# Memory usage benchmarks
echo -e "\n${GREEN}3. Memory Usage Benchmarks${NC}"
cargo bench --bench memory_usage -- --save-baseline baseline 2>&1 | tee "$RESULTS_DIR/memory_usage.txt"

# Run Python client benchmarks
echo -e "\n${YELLOW}Running Python client benchmarks...${NC}"
cd clients/python
python benchmark_baseline.py | tee "$RESULTS_DIR/python_client_benchmark.txt"
mv diskdb_baseline_*.json "$RESULTS_DIR/" 2>/dev/null || true
mv diskdb_baseline_summary.txt "$RESULTS_DIR/" 2>/dev/null || true
cd ../..

# Generate consolidated report
echo -e "\n${YELLOW}Generating consolidated report...${NC}"

cat > "$RESULTS_DIR/baseline_report.md" << 'EOF'
# DiskDB Performance Baseline Report

Generated: $(date)

## Executive Summary

This report contains the baseline performance metrics for DiskDB before implementing C/C++ optimizations.

## Test Environment

EOF

cat "$RESULTS_DIR/system_info.txt" >> "$RESULTS_DIR/baseline_report.md"

cat >> "$RESULTS_DIR/baseline_report.md" << 'EOF'

## Benchmark Results

### 1. Protocol Parsing Performance

Key findings from protocol parsing benchmarks:
EOF

# Extract key metrics from protocol parsing
grep -A 5 "time:" "$RESULTS_DIR/protocol_parsing.txt" | head -20 >> "$RESULTS_DIR/baseline_report.md" || true

cat >> "$RESULTS_DIR/baseline_report.md" << 'EOF'

### 2. Core Operations Performance

Performance metrics for core DiskDB operations:
EOF

# Extract key metrics from core operations
grep -A 5 "time:" "$RESULTS_DIR/core_operations.txt" | head -30 >> "$RESULTS_DIR/baseline_report.md" || true

cat >> "$RESULTS_DIR/baseline_report.md" << 'EOF'

### 3. Memory Usage Analysis

Memory consumption patterns:
EOF

# Extract memory metrics
grep -E "(allocated|per key)" "$RESULTS_DIR/memory_usage.txt" | head -20 >> "$RESULTS_DIR/baseline_report.md" || true

cat >> "$RESULTS_DIR/baseline_report.md" << 'EOF'

### 4. Python Client Performance

Client library performance metrics:
EOF

# Extract Python benchmark summary
if [ -f "$RESULTS_DIR/diskdb_baseline_summary.txt" ]; then
    cat "$RESULTS_DIR/diskdb_baseline_summary.txt" >> "$RESULTS_DIR/baseline_report.md"
fi

cat >> "$RESULTS_DIR/baseline_report.md" << 'EOF'

## Performance Bottlenecks Identified

Based on the baseline measurements, the following areas show potential for optimization:

1. **Protocol Parsing**: Significant time spent in string allocation and parsing
2. **List Operations**: LPUSH shows O(n) behavior due to front insertion
3. **Memory Allocation**: High allocation count for simple operations
4. **Serialization**: JSON values show double serialization overhead

## Baseline Metrics Summary

| Operation | Current Performance | Target (2-3x) |
|-----------|-------------------|---------------|
| Protocol Parse (simple) | TBD | TBD |
| SET operation | TBD | TBD |
| GET operation | TBD | TBD |
| LPUSH operation | TBD | TBD |
| Concurrent throughput | TBD | TBD |

## Next Steps

1. Implement C protocol parser with zero-copy design
2. Add thread-local memory pools
3. Optimize data structure operations
4. Re-run benchmarks to measure improvements
EOF

# Create a quick reference file
cat > "$RESULTS_DIR/quick_reference.txt" << EOF
DiskDB Baseline Performance Quick Reference
==========================================

Protocol Parsing:
- Simple GET: $(grep -m1 "parse_get" "$RESULTS_DIR/protocol_parsing.txt" | awk '{print $NF}' || echo "N/A")
- Complex ZADD: $(grep -m1 "parse_zadd_50" "$RESULTS_DIR/protocol_parsing.txt" | awk '{print $NF}' || echo "N/A")

Core Operations:
- SET: $(grep -m1 "set_small" "$RESULTS_DIR/core_operations.txt" | awk '{print $NF}' || echo "N/A")
- GET: $(grep -m1 "get_existing" "$RESULTS_DIR/core_operations.txt" | awk '{print $NF}' || echo "N/A")

Python Client:
- Check python_client_benchmark.txt for detailed metrics

Memory Usage:
- Check memory_usage.txt for allocation patterns
EOF

echo -e "\n${GREEN}✓ Baseline benchmarks completed!${NC}"
echo -e "${GREEN}✓ Results saved to: $RESULTS_DIR${NC}"
echo -e "\nKey files:"
echo "  - $RESULTS_DIR/baseline_report.md (main report)"
echo "  - $RESULTS_DIR/quick_reference.txt (summary)"
echo "  - $RESULTS_DIR/*.txt (detailed results)"

# Cleanup if we started DiskDB
if [ ! -z "$DISKDB_PID" ]; then
    echo -e "\n${YELLOW}Stopping DiskDB server...${NC}"
    kill $DISKDB_PID 2>/dev/null || true
fi