package com.example;

import com.google.common.collect.ImmutableList;

/**
 * Minimal Java application demonstrating BazBOM usage.
 */
public class App {
    public static void main(String[] args) {
        System.out.println("Hello from BazBOM example!");
        
        // Use Guava to demonstrate dependency
        ImmutableList<String> features = ImmutableList.of(
            "SBOM Generation",
            "SCA Scanning",
            "SARIF Reports",
            "GitHub Integration"
        );
        
        System.out.println("\nBazBOM Features:");
        for (String feature : features) {
            System.out.println("  - " + feature);
        }
    }
}
