use bazbom_java_reachability::analyze_java_project;
use std::path::Path;

fn main() {
    let report = analyze_java_project(Path::new("/tmp/java-reachability-test")).unwrap();
    
    println!("Java Reachability Analysis Results:");
    println!("=====================================");
    println!("Total methods: {}", report.all_functions.len());
    println!("Entrypoints: {}", report.entrypoints.len());
    println!("Reachable: {}", report.reachable_functions.len());
    println!("Unreachable: {}", report.unreachable_functions.len());
    println!();
    
    println!("Methods found:");
    for (id, method) in &report.all_functions {
        println!("  {} - {} (entrypoint: {}, reachable: {})", 
            id, 
            method.name,
            method.is_entrypoint,
            method.reachable
        );
    }
    
    println!();
    println!("Entrypoints:");
    for ep in &report.entrypoints {
        println!("  {}", ep);
    }
}
