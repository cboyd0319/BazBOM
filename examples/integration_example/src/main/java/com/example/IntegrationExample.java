package com.example;

import com.google.common.collect.ImmutableList;

/**
 * BazBOM Integration Example
 * 
 * This simple application demonstrates how BazBOM analyzes a Maven project
 * and generates comprehensive security intelligence.
 */
public class IntegrationExample {
    public static void main(String[] args) {
        // Example usage of Guava library
        ImmutableList<String> features = ImmutableList.of(
            "SBOM Generation (SPDX 2.3 + CycloneDX 1.5)",
            "SCA Analysis (OSV/NVD/GHSA)",
            "Optional Semgrep Pattern Analysis",
            "Optional CodeQL Dataflow Analysis",
            "deps.dev Enrichment",
            "OpenRewrite Autofix Recipes",
            "Unified SARIF 2.1.0 Output"
        );

        System.out.println("BazBOM Integration Features:");
        for (int i = 0; i < features.size(); i++) {
            System.out.println((i + 1) + ". " + features.get(i));
        }

        System.out.println("\nRun 'bazbom scan .' to analyze this project!");
    }
}
