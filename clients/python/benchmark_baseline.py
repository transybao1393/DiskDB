#!/usr/bin/env python3
"""
DiskDB Python Client Performance Baseline Benchmark

This script measures the baseline performance of the DiskDB Python client
for comparison with future optimizations.
"""

import time
import statistics
import json
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime
import sys
import os

# Add the current directory to path to import diskdb
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    from diskdb import DiskDB
except ImportError:
    print("Error: diskdb module not found. Please ensure it's in the same directory.")
    sys.exit(1)

class PerformanceBenchmark:
    def __init__(self, host='localhost', port=6380):
        self.db = DiskDB(host, port)
        self.results = {}
    
    def measure_time(self, func, iterations=1000):
        """Measure execution time for a function over multiple iterations"""
        times = []
        for _ in range(iterations):
            start = time.perf_counter()
            func()
            end = time.perf_counter()
            times.append((end - start) * 1000)  # Convert to milliseconds
        
        return {
            'mean': statistics.mean(times),
            'median': statistics.median(times),
            'stdev': statistics.stdev(times) if len(times) > 1 else 0,
            'min': min(times),
            'max': max(times),
            'p95': statistics.quantiles(times, n=20)[18] if len(times) > 20 else max(times),
            'p99': statistics.quantiles(times, n=100)[98] if len(times) > 100 else max(times),
        }
    
    def bench_string_operations(self):
        """Benchmark string operations"""
        print("\n=== String Operations ===")
        
        # SET benchmark
        def set_op():
            self.db.set("bench_key", "bench_value")
        
        set_results = self.measure_time(set_op, 10000)
        ops_per_sec = 1000 / set_results['mean']
        print(f"SET: {ops_per_sec:.0f} ops/sec (mean: {set_results['mean']:.3f}ms, p99: {set_results['p99']:.3f}ms)")
        self.results['string_set'] = set_results
        
        # GET benchmark
        self.db.set("get_key", "get_value")
        def get_op():
            self.db.get("get_key")
        
        get_results = self.measure_time(get_op, 10000)
        ops_per_sec = 1000 / get_results['mean']
        print(f"GET: {ops_per_sec:.0f} ops/sec (mean: {get_results['mean']:.3f}ms, p99: {get_results['p99']:.3f}ms)")
        self.results['string_get'] = get_results
        
        # INCR benchmark
        self.db.set("counter", "0")
        def incr_op():
            self.db.incr("counter")
        
        incr_results = self.measure_time(incr_op, 5000)
        ops_per_sec = 1000 / incr_results['mean']
        print(f"INCR: {ops_per_sec:.0f} ops/sec (mean: {incr_results['mean']:.3f}ms, p99: {incr_results['p99']:.3f}ms)")
        self.results['string_incr'] = incr_results
    
    def bench_list_operations(self):
        """Benchmark list operations"""
        print("\n=== List Operations ===")
        
        # LPUSH benchmark
        def lpush_op():
            self.db.lpush("bench_list", "item")
        
        lpush_results = self.measure_time(lpush_op, 5000)
        ops_per_sec = 1000 / lpush_results['mean']
        print(f"LPUSH: {ops_per_sec:.0f} ops/sec (mean: {lpush_results['mean']:.3f}ms, p99: {lpush_results['p99']:.3f}ms)")
        self.results['list_lpush'] = lpush_results
        
        # RPUSH benchmark
        def rpush_op():
            self.db.rpush("bench_list2", "item")
        
        rpush_results = self.measure_time(rpush_op, 5000)
        ops_per_sec = 1000 / rpush_results['mean']
        print(f"RPUSH: {ops_per_sec:.0f} ops/sec (mean: {rpush_results['mean']:.3f}ms, p99: {rpush_results['p99']:.3f}ms)")
        self.results['list_rpush'] = rpush_results
        
        # LRANGE benchmark on list with 100 items
        self.db.delete("range_list")
        for i in range(100):
            self.db.rpush("range_list", f"item_{i}")
        
        def lrange_op():
            self.db.lrange("range_list", 0, -1)
        
        lrange_results = self.measure_time(lrange_op, 1000)
        ops_per_sec = 1000 / lrange_results['mean']
        print(f"LRANGE (100 items): {ops_per_sec:.0f} ops/sec (mean: {lrange_results['mean']:.3f}ms, p99: {lrange_results['p99']:.3f}ms)")
        self.results['list_lrange_100'] = lrange_results
    
    def bench_set_operations(self):
        """Benchmark set operations"""
        print("\n=== Set Operations ===")
        
        # SADD benchmark
        def sadd_op():
            self.db.sadd("bench_set", "member1", "member2")
        
        sadd_results = self.measure_time(sadd_op, 5000)
        ops_per_sec = 1000 / sadd_results['mean']
        print(f"SADD: {ops_per_sec:.0f} ops/sec (mean: {sadd_results['mean']:.3f}ms, p99: {sadd_results['p99']:.3f}ms)")
        self.results['set_sadd'] = sadd_results
        
        # SISMEMBER benchmark on set with 1000 members
        self.db.delete("large_set")
        for i in range(1000):
            self.db.sadd("large_set", f"member_{i}")
        
        def sismember_op():
            self.db.sismember("large_set", "member_500")
        
        sismember_results = self.measure_time(sismember_op, 5000)
        ops_per_sec = 1000 / sismember_results['mean']
        print(f"SISMEMBER (1000 members): {ops_per_sec:.0f} ops/sec (mean: {sismember_results['mean']:.3f}ms, p99: {sismember_results['p99']:.3f}ms)")
        self.results['set_sismember_1000'] = sismember_results
    
    def bench_hash_operations(self):
        """Benchmark hash operations"""
        print("\n=== Hash Operations ===")
        
        # HSET benchmark
        def hset_op():
            self.db.hset("bench_hash", "field", "value")
        
        hset_results = self.measure_time(hset_op, 5000)
        ops_per_sec = 1000 / hset_results['mean']
        print(f"HSET: {ops_per_sec:.0f} ops/sec (mean: {hset_results['mean']:.3f}ms, p99: {hset_results['p99']:.3f}ms)")
        self.results['hash_hset'] = hset_results
        
        # HGETALL benchmark on hash with 100 fields
        self.db.delete("large_hash")
        for i in range(100):
            self.db.hset("large_hash", f"field_{i}", f"value_{i}")
        
        def hgetall_op():
            self.db.hgetall("large_hash")
        
        hgetall_results = self.measure_time(hgetall_op, 1000)
        ops_per_sec = 1000 / hgetall_results['mean']
        print(f"HGETALL (100 fields): {ops_per_sec:.0f} ops/sec (mean: {hgetall_results['mean']:.3f}ms, p99: {hgetall_results['p99']:.3f}ms)")
        self.results['hash_hgetall_100'] = hgetall_results
    
    def bench_sorted_set_operations(self):
        """Benchmark sorted set operations"""
        print("\n=== Sorted Set Operations ===")
        
        # ZADD benchmark
        def zadd_op():
            self.db.zadd("bench_zset", {"member": 1.0})
        
        zadd_results = self.measure_time(zadd_op, 5000)
        ops_per_sec = 1000 / zadd_results['mean']
        print(f"ZADD: {ops_per_sec:.0f} ops/sec (mean: {zadd_results['mean']:.3f}ms, p99: {zadd_results['p99']:.3f}ms)")
        self.results['zset_zadd'] = zadd_results
        
        # ZRANGE benchmark on sorted set with 1000 members
        self.db.delete("large_zset")
        members = {f"member_{i}": float(i) for i in range(1000)}
        self.db.zadd("large_zset", members)
        
        def zrange_op():
            self.db.zrange("large_zset", 0, 99)  # Top 100
        
        zrange_results = self.measure_time(zrange_op, 1000)
        ops_per_sec = 1000 / zrange_results['mean']
        print(f"ZRANGE (top 100 of 1000): {ops_per_sec:.0f} ops/sec (mean: {zrange_results['mean']:.3f}ms, p99: {zrange_results['p99']:.3f}ms)")
        self.results['zset_zrange_100_of_1000'] = zrange_results
    
    def bench_json_operations(self):
        """Benchmark JSON operations"""
        print("\n=== JSON Operations ===")
        
        # JSON.SET benchmark
        test_json = {
            "user": {
                "name": "Alice",
                "age": 30,
                "scores": [100, 95, 87]
            }
        }
        
        def json_set_op():
            self.db.json_set("bench_json", "$", test_json)
        
        json_set_results = self.measure_time(json_set_op, 2000)
        ops_per_sec = 1000 / json_set_results['mean']
        print(f"JSON.SET: {ops_per_sec:.0f} ops/sec (mean: {json_set_results['mean']:.3f}ms, p99: {json_set_results['p99']:.3f}ms)")
        self.results['json_set'] = json_set_results
        
        # JSON.GET benchmark
        self.db.json_set("json_doc", "$", test_json)
        
        def json_get_op():
            self.db.json_get("json_doc", "$")
        
        json_get_results = self.measure_time(json_get_op, 2000)
        ops_per_sec = 1000 / json_get_results['mean']
        print(f"JSON.GET: {ops_per_sec:.0f} ops/sec (mean: {json_get_results['mean']:.3f}ms, p99: {json_get_results['p99']:.3f}ms)")
        self.results['json_get'] = json_get_results
    
    def bench_concurrent_operations(self):
        """Benchmark concurrent operations"""
        print("\n=== Concurrent Operations ===")
        
        thread_counts = [1, 2, 4, 8]
        operations_per_thread = 1000
        
        for threads in thread_counts:
            def worker(thread_id):
                latencies = []
                for i in range(operations_per_thread):
                    start = time.perf_counter()
                    self.db.set(f"concurrent_{thread_id}_{i}", "value")
                    self.db.get(f"concurrent_{thread_id}_{i}")
                    latencies.append((time.perf_counter() - start) * 1000)
                return latencies
            
            start_time = time.perf_counter()
            all_latencies = []
            
            with ThreadPoolExecutor(max_workers=threads) as executor:
                futures = [executor.submit(worker, i) for i in range(threads)]
                for future in as_completed(futures):
                    all_latencies.extend(future.result())
            
            duration = time.perf_counter() - start_time
            total_ops = threads * operations_per_thread * 2  # SET + GET
            throughput = total_ops / duration
            
            stats = {
                'threads': threads,
                'throughput_ops_sec': throughput,
                'mean_latency': statistics.mean(all_latencies),
                'p99_latency': statistics.quantiles(all_latencies, n=100)[98] if len(all_latencies) > 100 else max(all_latencies),
            }
            
            print(f"{threads} threads: {throughput:.0f} ops/sec (mean: {stats['mean_latency']:.3f}ms, p99: {stats['p99_latency']:.3f}ms)")
            self.results[f'concurrent_{threads}_threads'] = stats
    
    def bench_large_value_operations(self):
        """Benchmark operations with large values"""
        print("\n=== Large Value Operations ===")
        
        sizes = [1024, 10240, 102400]  # 1KB, 10KB, 100KB
        
        for size in sizes:
            value = "x" * size
            
            def set_large_op():
                self.db.set(f"large_key_{size}", value)
            
            set_results = self.measure_time(set_large_op, 100)
            ops_per_sec = 1000 / set_results['mean']
            size_kb = size // 1024
            print(f"SET {size_kb}KB: {ops_per_sec:.0f} ops/sec (mean: {set_results['mean']:.3f}ms)")
            self.results[f'large_value_set_{size_kb}kb'] = set_results
    
    def save_results(self):
        """Save benchmark results to file"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"diskdb_baseline_benchmark_{timestamp}.json"
        
        report = {
            'timestamp': timestamp,
            'system_info': {
                'python_version': sys.version,
                'platform': sys.platform,
            },
            'results': self.results
        }
        
        with open(filename, 'w') as f:
            json.dump(report, f, indent=2)
        
        print(f"\nResults saved to: {filename}")
        
        # Also create a summary report
        summary_file = "diskdb_baseline_summary.txt"
        with open(summary_file, 'w') as f:
            f.write(f"DiskDB Python Client Baseline Performance Report\n")
            f.write(f"Generated: {timestamp}\n")
            f.write("=" * 60 + "\n\n")
            
            f.write("Key Performance Metrics:\n")
            f.write("-" * 30 + "\n")
            
            # String operations
            if 'string_set' in self.results:
                set_ops = 1000 / self.results['string_set']['mean']
                f.write(f"SET: {set_ops:.0f} ops/sec\n")
            
            if 'string_get' in self.results:
                get_ops = 1000 / self.results['string_get']['mean']
                f.write(f"GET: {get_ops:.0f} ops/sec\n")
            
            # Concurrent performance
            if 'concurrent_8_threads' in self.results:
                throughput = self.results['concurrent_8_threads']['throughput_ops_sec']
                f.write(f"\nConcurrent (8 threads): {throughput:.0f} ops/sec\n")
            
            f.write("\nDetailed results saved in JSON format\n")
        
        print(f"Summary saved to: {summary_file}")
    
    def run_all_benchmarks(self):
        """Run all benchmarks"""
        print("DiskDB Python Client Baseline Performance Benchmark")
        print("=" * 50)
        
        try:
            self.bench_string_operations()
            self.bench_list_operations()
            self.bench_set_operations()
            self.bench_hash_operations()
            self.bench_sorted_set_operations()
            self.bench_json_operations()
            self.bench_large_value_operations()
            self.bench_concurrent_operations()
            
            self.save_results()
            
        except Exception as e:
            print(f"\nError during benchmark: {e}")
            print("Make sure DiskDB server is running on localhost:6380")

if __name__ == "__main__":
    benchmark = PerformanceBenchmark()
    benchmark.run_all_benchmarks()