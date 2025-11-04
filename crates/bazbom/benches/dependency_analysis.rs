//! Performance benchmarks for dependency analysis
//!
//! Measures scanning performance with different project sizes:
//! - Small: 100 dependencies
//! - Medium: 1,000 dependencies
//! - Large: 10,000 dependencies
//! - Huge: 50,000 dependencies

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;

/// Generate a mock dependency graph with N dependencies
fn generate_mock_dependencies(count: usize) -> HashMap<String, Vec<String>> {
    let mut deps = HashMap::new();
    
    for i in 0..count {
        let pkg_name = format!("org.example:package-{}", i);
        
        // Each package depends on 3-5 other packages
        let num_deps = (i % 3) + 3;
        let mut package_deps = Vec::new();
        
        for j in 0..num_deps {
            let dep_idx = (i + j + 1) % count;
            package_deps.push(format!("org.example:package-{}", dep_idx));
        }
        
        deps.insert(pkg_name, package_deps);
    }
    
    deps
}

/// Benchmark dependency graph traversal
fn bench_graph_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_traversal");
    
    for size in &[100, 1_000, 10_000] {
        let deps = generate_mock_dependencies(*size);
        
        group.bench_with_input(
            BenchmarkId::new("traverse", size),
            &deps,
            |b, deps| {
                b.iter(|| {
                    // Traverse all dependencies
                    let mut visited = std::collections::HashSet::new();
                    for (pkg, _) in deps {
                        if !visited.contains(pkg) {
                            visited.insert(pkg.clone());
                        }
                    }
                    visited.len()
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark dependency resolution
fn bench_dependency_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_resolution");
    
    for size in &[100, 1_000, 10_000] {
        let deps = generate_mock_dependencies(*size);
        
        group.bench_with_input(
            BenchmarkId::new("resolve", size),
            &deps,
            |b, deps| {
                b.iter(|| {
                    // Resolve all transitive dependencies
                    let mut resolved = HashMap::new();
                    for (pkg, pkg_deps) in deps {
                        let mut transitive = std::collections::HashSet::new();
                        for dep in pkg_deps {
                            transitive.insert(dep.clone());
                            if let Some(sub_deps) = deps.get(dep) {
                                for sub_dep in sub_deps {
                                    transitive.insert(sub_dep.clone());
                                }
                            }
                        }
                        resolved.insert(pkg.clone(), transitive);
                    }
                    resolved.len()
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark parallel processing
fn bench_parallel_processing(c: &mut Criterion) {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let mut group = c.benchmark_group("parallel_processing");
    
    for size in &[1_000, 10_000] {
        let deps = generate_mock_dependencies(*size);
        
        group.bench_with_input(
            BenchmarkId::new("parallel", size),
            &deps,
            |b, deps| {
                b.iter(|| {
                    // Process dependencies in parallel
                    let packages: Vec<_> = deps.keys().cloned().collect();
                    let num_threads = num_cpus::get();
                    let chunk_size = (packages.len() + num_threads - 1) / num_threads;
                    
                    let results = Arc::new(Mutex::new(Vec::new()));
                    let mut handles = vec![];
                    
                    for chunk in packages.chunks(chunk_size) {
                        let chunk = chunk.to_vec();
                        let results = Arc::clone(&results);
                        
                        let handle = thread::spawn(move || {
                            let mut local_results = Vec::new();
                            for pkg in chunk {
                                // Simulate some processing
                                local_results.push(pkg.len());
                            }
                            results.lock().unwrap().extend(local_results);
                        });
                        
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                    
                    results.lock().unwrap().len()
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark caching performance
fn bench_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("caching");
    
    for size in &[100, 1_000, 10_000] {
        let deps = generate_mock_dependencies(*size);
        
        group.bench_with_input(
            BenchmarkId::new("with_cache", size),
            &deps,
            |b, deps| {
                let mut cache = HashMap::new();
                
                b.iter(|| {
                    for (pkg, pkg_deps) in deps {
                        // Check cache first
                        if let Some(cached) = cache.get(pkg) {
                            let _ = cached;
                        } else {
                            // Calculate and cache
                            let result = pkg_deps.len();
                            cache.insert(pkg.clone(), result);
                        }
                    }
                    cache.len()
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_graph_traversal,
    bench_dependency_resolution,
    bench_parallel_processing,
    bench_caching
);
criterion_main!(benches);
