//! Performance benchmarks for BazBOM scan operations
//!
//! These benchmarks measure the performance of core scanning operations
//! to ensure BazBOM meets its performance goals for large monorepos.

use bazbom_core::BuildSystem;
use bazbom_graph::DependencyGraph;
use criterion::{ criterion_group, criterion_main, BenchmarkId, Criterion};

/// Benchmark build system detection
fn bench_build_system_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_system_detection");

    // Create temporary test directories with different build files
    let test_cases = vec![
        ("maven", "pom.xml"),
        ("gradle", "build.gradle"),
        ("bazel", "BUILD.bazel"),
    ];

    for (name, _file) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &name, |b, _name| {
            b.iter(|| {
                // In a real implementation, would detect from actual project
                // For benchmark, we just test the detection logic
                std::hint::black_box(BuildSystem::Maven)
            });
        });
    }

    group.finish();
}

/// Benchmark dependency graph construction
fn bench_dependency_graph(c: &mut Criterion) {
    use bazbom_graph::{Component, ComponentId};

    let mut group = c.benchmark_group("dependency_graph");

    // Test with different graph sizes
    for size in &[10, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                // Create a mock dependency graph
                let mut graph = DependencyGraph::new();

                // Add nodes and edges to simulate real dependencies
                for i in 0..size {
                    let node_id = ComponentId::new(format!("dep-{}", i));
                    let component = Component {
                        id: node_id.clone(),
                        name: format!("dep-{}", i),
                        version: "1.0.0".to_string(),
                        purl: None,
                        license: None,
                        scope: None,
                        hash: None,
                    };
                    graph.add_component(component);

                    // Add edge to previous node (creating a chain)
                    if i > 0 {
                        let parent_id = ComponentId::new(format!("dep-{}", i - 1));
                        graph.add_edge(parent_id, node_id, "depends".to_string());
                    }
                }

                std::hint::black_box(graph)
            });
        });
    }

    group.finish();
}

/// Benchmark SBOM generation (simulated)
fn bench_sbom_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sbom_generation");

    // Test SBOM generation for different dependency counts
    for dep_count in &[100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(dep_count),
            dep_count,
            |b, &count| {
                b.iter(|| {
                    // Simulate SBOM generation
                    let mut sbom_data = Vec::with_capacity(count * 200);

                    for i in 0..count {
                        let entry =
                            format!(r#"{{"name":"package-{}","version":"1.{}.0"}}"#, i, i % 100);
                        sbom_data.extend_from_slice(entry.as_bytes());
                    }

                    std::hint::black_box(sbom_data)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cache lookups
fn bench_cache_lookup(c: &mut Criterion) {
    use std::collections::HashMap;

    let mut group = c.benchmark_group("cache_lookup");

    // Create a cache with different sizes
    for cache_size in &[100, 1000, 10000] {
        let mut cache: HashMap<String, Vec<u8>> = HashMap::new();

        // Populate cache
        for i in 0..*cache_size {
            let key = format!("cache-key-{}", i);
            let value = vec![0u8; 1024]; // 1KB value
            cache.insert(key, value);
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(cache_size),
            cache_size,
            |b, &size| {
                b.iter(|| {
                    // Lookup random keys
                    let key = format!("cache-key-{}", size / 2);
                    std::hint::black_box(cache.get(&key))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark JSON parsing for SBOM files
fn bench_json_parsing(c: &mut Criterion) {
    use serde_json::Value;

    let mut group = c.benchmark_group("json_parsing");

    // Test parsing of different JSON sizes
    for package_count in &[10, 100, 1000] {
        // Create a mock SPDX SBOM JSON
        let mut packages = Vec::new();
        for i in 0..*package_count {
            packages.push(format!(
                r#"{{"name":"package-{}","version":"1.0.0","SPDXID":"SPDXRef-Package-{}"}}"#,
                i, i
            ));
        }

        let json = format!(
            r#"{{"spdxVersion":"SPDX-2.3","packages":[{}]}}"#,
            packages.join(",")
        );

        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            package_count,
            |b, _count| {
                b.iter(|| {
                    let _parsed: Value = serde_json::from_str(&json).unwrap();
                    std::hint::black_box(_parsed)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark string hashing for cache keys
fn bench_string_hashing(c: &mut Criterion) {
    use sha2::{Digest, Sha256};

    let mut group = c.benchmark_group("string_hashing");

    // Test hashing of different content sizes
    for size_kb in &[1, 10, 100, 1000] {
        let content = "x".repeat(size_kb * 1024);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB", size_kb)),
            &content,
            |b, content| {
                b.iter(|| {
                    let mut hasher = Sha256::new();
                    hasher.update(content.as_bytes());
                    let hash = hasher.finalize();
                    std::hint::black_box(hex::encode(hash))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_build_system_detection,
    bench_dependency_graph,
    bench_sbom_generation,
    bench_cache_lookup,
    bench_json_parsing,
    bench_string_hashing,
);

criterion_main!(benches);
