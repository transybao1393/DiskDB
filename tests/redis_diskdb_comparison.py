#!/usr/bin/env python3
"""
Performance comparison between Redis and DiskDB
"""

import time
import redis
import statistics
import sys
sys.path.append('../clients/python')
from diskdb import DiskDB
import json
import subprocess

# Configuration
NUM_OPERATIONS = 10000
KEY_PREFIX = "bench_key_"
VALUE_SIZE = 100  # bytes

def generate_value(size):
    """Generate a value of specific size"""
    return "x" * size

def clear_redis():
    """Clear Redis database"""
    r = redis.Redis(port=6379)
    r.flushdb()

def clear_diskdb():
    """Clear DiskDB database"""
    db = DiskDB(port=6380)
    # DiskDB doesn't support FLUSHDB, so manually delete test keys
    for i in range(NUM_OPERATIONS):
        db.delete(f"{KEY_PREFIX}{i}")
    db.delete("bench_list")

def benchmark_redis_set(num_ops):
    """Benchmark Redis SET operations"""
    r = redis.Redis(port=6379, decode_responses=True)
    value = generate_value(VALUE_SIZE)
    
    times = []
    for i in range(num_ops):
        start = time.perf_counter()
        r.set(f"{KEY_PREFIX}{i}", value)
        end = time.perf_counter()
        times.append(end - start)
    
    return times

def benchmark_diskdb_set(num_ops):
    """Benchmark DiskDB SET operations"""
    db = DiskDB(port=6380)
    value = generate_value(VALUE_SIZE)
    
    times = []
    for i in range(num_ops):
        start = time.perf_counter()
        db.set(f"{KEY_PREFIX}{i}", value)
        end = time.perf_counter()
        times.append(end - start)
    
    return times

def benchmark_redis_get(num_ops):
    """Benchmark Redis GET operations"""
    r = redis.Redis(port=6379, decode_responses=True)
    
    times = []
    for i in range(num_ops):
        start = time.perf_counter()
        r.get(f"{KEY_PREFIX}{i}")
        end = time.perf_counter()
        times.append(end - start)
    
    return times

def benchmark_diskdb_get(num_ops):
    """Benchmark DiskDB GET operations"""
    db = DiskDB(port=6380)
    
    times = []
    for i in range(num_ops):
        start = time.perf_counter()
        db.get(f"{KEY_PREFIX}{i}")
        end = time.perf_counter()
        times.append(end - start)
    
    return times

def benchmark_redis_list(num_ops):
    """Benchmark Redis List operations"""
    r = redis.Redis(port=6379, decode_responses=True)
    
    # LPUSH operations
    lpush_times = []
    for i in range(num_ops // 2):
        start = time.perf_counter()
        r.lpush("bench_list", f"item_{i}")
        end = time.perf_counter()
        lpush_times.append(end - start)
    
    # LPOP operations
    lpop_times = []
    for i in range(num_ops // 2):
        start = time.perf_counter()
        r.lpop("bench_list")
        end = time.perf_counter()
        lpop_times.append(end - start)
    
    return lpush_times + lpop_times

def benchmark_diskdb_list(num_ops):
    """Benchmark DiskDB List operations"""
    db = DiskDB(port=6380)
    
    # LPUSH operations
    lpush_times = []
    for i in range(num_ops // 2):
        start = time.perf_counter()
        db.lpush("bench_list", f"item_{i}")
        end = time.perf_counter()
        lpush_times.append(end - start)
    
    # LPOP operations
    lpop_times = []
    for i in range(num_ops // 2):
        start = time.perf_counter()
        db.lpop("bench_list")
        end = time.perf_counter()
        lpop_times.append(end - start)
    
    return lpush_times + lpop_times

def calculate_stats(times):
    """Calculate performance statistics"""
    total_time = sum(times)
    ops_per_sec = len(times) / total_time
    avg_latency_ms = (statistics.mean(times) * 1000)
    p99_latency_ms = (sorted(times)[int(len(times) * 0.99)] * 1000)
    
    return {
        'total_time': total_time,
        'ops_per_sec': ops_per_sec,
        'avg_latency_ms': avg_latency_ms,
        'p99_latency_ms': p99_latency_ms
    }

def print_comparison_table(redis_stats, diskdb_stats, operation):
    """Print a formatted comparison table"""
    print(f"\n{'='*60}")
    print(f"{operation} Operation Comparison ({NUM_OPERATIONS} operations)")
    print(f"{'='*60}")
    print(f"{'Metric':<25} {'Redis':<15} {'DiskDB':<15} {'Difference':<15}")
    print(f"{'-'*60}")
    
    # Operations per second
    redis_ops = redis_stats['ops_per_sec']
    diskdb_ops = diskdb_stats['ops_per_sec']
    ops_diff = ((diskdb_ops - redis_ops) / redis_ops) * 100
    print(f"{'Operations/sec':<25} {redis_ops:>14,.0f} {diskdb_ops:>14,.0f} {ops_diff:>+14.1f}%")
    
    # Average latency
    redis_avg = redis_stats['avg_latency_ms']
    diskdb_avg = diskdb_stats['avg_latency_ms']
    avg_diff = ((diskdb_avg - redis_avg) / redis_avg) * 100
    print(f"{'Avg Latency (ms)':<25} {redis_avg:>14.3f} {diskdb_avg:>14.3f} {avg_diff:>+14.1f}%")
    
    # P99 latency
    redis_p99 = redis_stats['p99_latency_ms']
    diskdb_p99 = diskdb_stats['p99_latency_ms']
    p99_diff = ((diskdb_p99 - redis_p99) / redis_p99) * 100
    print(f"{'P99 Latency (ms)':<25} {redis_p99:>14.3f} {diskdb_p99:>14.3f} {p99_diff:>+14.1f}%")
    
    # Total time
    redis_total = redis_stats['total_time']
    diskdb_total = diskdb_stats['total_time']
    total_diff = ((diskdb_total - redis_total) / redis_total) * 100
    print(f"{'Total Time (s)':<25} {redis_total:>14.3f} {diskdb_total:>14.3f} {total_diff:>+14.1f}%")

def main():
    print("Redis vs DiskDB Performance Comparison")
    print("=====================================")
    
    # Check if Redis is running
    try:
        r = redis.Redis(port=6379)
        r.ping()
    except:
        print("ERROR: Redis is not running on port 6379")
        print("Please start Redis with: redis-server")
        sys.exit(1)
    
    # Check if DiskDB is running
    try:
        db = DiskDB(port=6380)
        # DiskDB doesn't support PING, so test with SET/GET
        db.set("_test_connection", "test")
        db.delete("_test_connection")
    except Exception as e:
        print(f"ERROR: Cannot connect to DiskDB on port 6380: {e}")
        print("Please start DiskDB with: cargo run --release")
        sys.exit(1)
    
    # Clear both databases
    print("\nClearing databases...")
    clear_redis()
    clear_diskdb()
    
    # SET Operation Benchmark
    print("\nRunning SET benchmarks...")
    redis_set_times = benchmark_redis_set(NUM_OPERATIONS)
    diskdb_set_times = benchmark_diskdb_set(NUM_OPERATIONS)
    
    redis_set_stats = calculate_stats(redis_set_times)
    diskdb_set_stats = calculate_stats(diskdb_set_times)
    print_comparison_table(redis_set_stats, diskdb_set_stats, "SET")
    
    # GET Operation Benchmark
    print("\nRunning GET benchmarks...")
    redis_get_times = benchmark_redis_get(NUM_OPERATIONS)
    diskdb_get_times = benchmark_diskdb_get(NUM_OPERATIONS)
    
    redis_get_stats = calculate_stats(redis_get_times)
    diskdb_get_stats = calculate_stats(diskdb_get_times)
    print_comparison_table(redis_get_stats, diskdb_get_stats, "GET")
    
    # List Operation Benchmark
    print("\nRunning List operation benchmarks...")
    clear_redis()
    clear_diskdb()
    
    redis_list_times = benchmark_redis_list(NUM_OPERATIONS)
    diskdb_list_times = benchmark_diskdb_list(NUM_OPERATIONS)
    
    redis_list_stats = calculate_stats(redis_list_times)
    diskdb_list_stats = calculate_stats(diskdb_list_times)
    print_comparison_table(redis_list_stats, diskdb_list_stats, "LIST (LPUSH/LPOP)")
    
    # Summary
    print("\n" + "="*60)
    print("SUMMARY")
    print("="*60)
    
    # Calculate overall performance ratio
    redis_total_ops = (redis_set_stats['ops_per_sec'] + 
                      redis_get_stats['ops_per_sec'] + 
                      redis_list_stats['ops_per_sec']) / 3
    
    diskdb_total_ops = (diskdb_set_stats['ops_per_sec'] + 
                       diskdb_get_stats['ops_per_sec'] + 
                       diskdb_list_stats['ops_per_sec']) / 3
    
    performance_ratio = diskdb_total_ops / redis_total_ops
    
    print(f"\nAverage Operations/sec:")
    print(f"  Redis:  {redis_total_ops:>10,.0f} ops/sec")
    print(f"  DiskDB: {diskdb_total_ops:>10,.0f} ops/sec")
    print(f"\nDiskDB Performance: {performance_ratio:.2f}x Redis")
    
    # Key differences
    print("\nKey Observations:")
    print("- DiskDB provides automatic persistence to disk")
    print("- Redis operates primarily in memory")
    print("- DiskDB is optimized for mixed read/write workloads")
    print("- Performance varies based on operation type and data size")
    
    # Generate visual comparison
    print("\n" + "="*60)
    print("Visual Performance Comparison")
    print("="*60)
    
    # Create bar chart
    max_width = 40
    redis_bar_width = int((redis_total_ops / max(redis_total_ops, diskdb_total_ops)) * max_width)
    diskdb_bar_width = int((diskdb_total_ops / max(redis_total_ops, diskdb_total_ops)) * max_width)
    
    print(f"\nRedis:  |{'█' * redis_bar_width}{' ' * (max_width - redis_bar_width)}| {redis_total_ops:,.0f} ops/s")
    print(f"DiskDB: |{'█' * diskdb_bar_width}{' ' * (max_width - diskdb_bar_width)}| {diskdb_total_ops:,.0f} ops/s")

if __name__ == "__main__":
    main()