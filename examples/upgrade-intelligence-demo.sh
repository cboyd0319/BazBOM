#!/usr/bin/env bash
# Demo script showing BazBOM's upgrade intelligence feature

set -e

echo "=============================================="
echo "BazBOM Upgrade Intelligence Demo"
echo "=============================================="
echo

# Example 1: Safe patch version upgrade
echo "Example 1: Safe Patch Version Upgrade"
echo "--------------------------------------"
echo "Analyzing: log4j-core 2.17.0 → 2.17.1"
echo
bazbom fix org.apache.logging.log4j:log4j-core --explain || true
echo
echo "Press Enter to continue..."
read -r

# Example 2: Minor version with transitive changes
echo
echo "Example 2: Minor Version with Transitive Changes"
echo "-------------------------------------------------"
echo "Analyzing: log4j-core 2.17.0 → 2.20.0"
echo
bazbom fix org.apache.logging.log4j:log4j-core --explain || true
echo
echo "Press Enter to continue..."
read -r

# Example 3: Major version upgrade
echo
echo "Example 3: Major Version Upgrade"
echo "---------------------------------"
echo "Analyzing: Spring Boot 2.7.0 → 3.2.0"
echo
bazbom fix org.springframework.boot:spring-boot-starter-web --explain || true
echo
echo "Demo complete!"
echo
echo "Try it yourself:"
echo "  bazbom fix <package-name> --explain"
echo
echo "Examples:"
echo "  bazbom fix com.google.guava:guava --explain"
echo "  bazbom fix org.hibernate:hibernate-core --explain"
echo "  bazbom fix com.fasterxml.jackson.core:jackson-databind --explain"
