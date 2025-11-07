//! Performance benchmarks for Bazel scan operations
//!
//! Measures the performance of key Bazel scanning operations:
//! - Dependency graph parsing from maven_install.json
//! - SPDX document generation
//! - Bazel query optimization and caching
//! - Component serialization and deserialization

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;

/// Benchmark parsing maven_install.json files of various sizes
fn bench_maven_install_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("maven_install_parsing");

    for size in &[10, 50, 100, 500] {
        let json_content = generate_maven_install_json(*size);

        group.bench_with_input(BenchmarkId::new("parse", size), &json_content, |b, json| {
            b.iter(|| {
                // Parse JSON
                let data: serde_json::Value = serde_json::from_str(json).unwrap();
                std::hint::black_box(data);
            });
        });
    }

    group.finish();
}

/// Benchmark SPDX document generation from dependency graphs
fn bench_spdx_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("spdx_generation");

    for size in &[10, 50, 100, 500] {
        let components = generate_mock_components(*size);

        group.bench_with_input(
            BenchmarkId::new("generate", size),
            &components,
            |b, components| {
                b.iter(|| {
                    // Generate SPDX document structure
                    let mut packages = Vec::new();
                    let mut relationships = Vec::new();

                    for (idx, _component) in components.iter().enumerate() {
                        let spdx_id = format!("Package-{}", idx);
                        packages.push(spdx_id.clone());
                        relationships.push(format!("DESCRIBES-{}", spdx_id));
                    }

                    std::hint::black_box((packages, relationships));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Bazel query caching performance
fn bench_query_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_caching");

    for cache_size in &[100, 1000, 5000] {
        // Create a cache with pre-populated entries
        let mut cache: HashMap<String, Vec<String>> = HashMap::new();
        for i in 0..*cache_size {
            let key = format!("query_{}", i);
            let value = vec![format!("target_{}", i), format!("target_{}", i + 1)];
            cache.insert(key, value);
        }

        group.bench_with_input(
            BenchmarkId::new("cache_hit", cache_size),
            &cache,
            |b, cache| {
                b.iter(|| {
                    // Simulate cache hits
                    let key = format!("query_{}", cache_size / 2);
                    let result = cache.get(&key);
                    std::hint::black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark component serialization and deserialization
fn bench_component_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_serialization");

    for size in &[10, 50, 100, 500] {
        let components = generate_mock_components(*size);
        let json_str = serde_json::to_string(&components).unwrap();

        group.bench_with_input(
            BenchmarkId::new("serialize", size),
            &components,
            |b, components| {
                b.iter(|| {
                    let json = serde_json::to_string(components).unwrap();
                    std::hint::black_box(json);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("deserialize", size),
            &json_str,
            |b, json_str| {
                b.iter(|| {
                    let components: Vec<MockComponent> = serde_json::from_str(json_str).unwrap();
                    std::hint::black_box(components);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark dependency graph traversal
fn bench_dependency_graph_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_traversal");

    for size in &[10, 50, 100, 500] {
        let graph = generate_dependency_graph(*size);

        group.bench_with_input(
            BenchmarkId::new("traverse_all", size),
            &graph,
            |b, graph| {
                b.iter(|| {
                    // Traverse all nodes in the graph
                    let mut visited = std::collections::HashSet::new();
                    for node in graph.keys() {
                        visited.insert(node.clone());
                    }
                    std::hint::black_box(visited.len());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("find_dependencies", size),
            &graph,
            |b, graph| {
                b.iter(|| {
                    // Find all dependencies for each node
                    let mut all_deps = Vec::new();
                    for (node, deps) in graph {
                        for dep in deps {
                            all_deps.push((node.clone(), dep.clone()));
                        }
                    }
                    std::hint::black_box(all_deps.len());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark PURL generation
fn bench_purl_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("purl_generation");

    for count in &[100, 500, 1000, 5000] {
        let components = generate_mock_component_data(*count);

        group.bench_with_input(
            BenchmarkId::new("generate", count),
            &components,
            |b, components| {
                b.iter(|| {
                    let purls: Vec<String> = components
                        .iter()
                        .map(|(group, artifact, version)| {
                            format!(
                                "pkg:maven/{}/{}@{}",
                                group.replace('.', "/"),
                                artifact,
                                version
                            )
                        })
                        .collect();
                    std::hint::black_box(purls);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parallel processing of dependencies
fn bench_parallel_processing(c: &mut Criterion) {
    use rayon::prelude::*;

    let mut group = c.benchmark_group("parallel_processing");

    for size in &[100, 1000, 5000] {
        let components = generate_mock_components(*size);

        group.bench_with_input(
            BenchmarkId::new("sequential", size),
            &components,
            |b, components| {
                b.iter(|| {
                    let results: Vec<_> = components.iter().map(simulate_processing).collect();
                    std::hint::black_box(results);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parallel", size),
            &components,
            |b, components| {
                b.iter(|| {
                    let results: Vec<_> = components.par_iter().map(simulate_processing).collect();
                    std::hint::black_box(results);
                });
            },
        );
    }

    group.finish();
}

// Helper functions and types

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MockComponent {
    name: String,
    group: String,
    version: String,
    purl: String,
    coordinates: String,
}

fn generate_maven_install_json(artifact_count: usize) -> String {
    let mut artifacts = serde_json::Map::new();
    let mut dependencies = serde_json::Map::new();

    for i in 0..artifact_count {
        let coord = format!("org.example:artifact-{}", i);
        let mut artifact_info = serde_json::Map::new();
        artifact_info.insert("version".to_string(), serde_json::json!("1.0.0"));

        let mut shasums = serde_json::Map::new();
        shasums.insert("jar".to_string(), serde_json::json!("abc123"));
        artifact_info.insert("shasums".to_string(), serde_json::json!(shasums));

        artifacts.insert(coord.clone(), serde_json::json!(artifact_info));

        // Add some dependencies
        if i > 0 {
            let dep_coord = format!("org.example:artifact-{}", i - 1);
            dependencies.insert(format!("{}:1.0.0", coord), serde_json::json!([dep_coord]));
        }
    }

    let json = serde_json::json!({
        "version": "2",
        "artifacts": artifacts,
        "dependencies": dependencies,
        "repositories": {}
    });

    serde_json::to_string(&json).unwrap()
}

fn generate_mock_components(count: usize) -> Vec<MockComponent> {
    (0..count)
        .map(|i| MockComponent {
            name: format!("artifact-{}", i),
            group: "org.example".to_string(),
            version: "1.0.0".to_string(),
            purl: format!("pkg:maven/org/example/artifact-{}@1.0.0", i),
            coordinates: format!("org.example:artifact-{}:1.0.0", i),
        })
        .collect()
}

fn generate_dependency_graph(size: usize) -> HashMap<String, Vec<String>> {
    let mut graph = HashMap::new();

    for i in 0..size {
        let node = format!("node-{}", i);
        let mut deps = Vec::new();

        // Each node depends on 2-3 other nodes
        let num_deps = (i % 2) + 2;
        for j in 0..num_deps {
            let dep_idx = (i + j + 1) % size;
            deps.push(format!("node-{}", dep_idx));
        }

        graph.insert(node, deps);
    }

    graph
}

fn generate_mock_component_data(count: usize) -> Vec<(String, String, String)> {
    (0..count)
        .map(|i| {
            (
                "org.example".to_string(),
                format!("artifact-{}", i),
                "1.0.0".to_string(),
            )
        })
        .collect()
}

fn simulate_processing(component: &MockComponent) -> usize {
    // Simulate some processing work
    component.name.len() + component.group.len() + component.version.len()
}

criterion_group!(
    benches,
    bench_maven_install_parsing,
    bench_spdx_generation,
    bench_query_caching,
    bench_component_serialization,
    bench_dependency_graph_traversal,
    bench_purl_generation,
    bench_parallel_processing
);
criterion_main!(benches);
