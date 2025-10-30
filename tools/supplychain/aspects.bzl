"""Bazel aspects for dependency discovery and SBOM generation.

This module provides Bazel aspects for extracting dependency information from
Java targets in Bazel projects. It supports rules_jvm_external (maven_install)
and extracts Maven coordinates, PURLs, checksums, and dependency relationships.
"""

load("@rules_java//java:defs.bzl", "JavaInfo")

# Immutable struct to represent package information (for use in depsets)
PackageInfo = provider(
    doc = "Immutable package information for SBOM",
    fields = {
        "name": "Package name",
        "group": "Maven group ID",
        "version": "Package version",
        "purl": "Package URL",
        "type": "Package type (e.g., maven)",
        "label": "Bazel target label (provenance)",
        "sha256": "SHA256 checksum if available",
        "is_direct": "Whether this is a direct dependency",
        "scope": "Dependency scope (compile, runtime, test)",
    },
)

SbomInfo = provider(
    doc = "Information about dependencies for SBOM generation",
    fields = {
        "packages": "Depset of PackageInfo structs",
        "target_label": "Label of the target",
    },
)

def _extract_maven_coordinates(target, ctx):
    """Extract Maven coordinates from a target if available.
    
    This function extracts dependency information from Bazel targets that represent
    Maven artifacts, particularly those managed by rules_jvm_external.
    
    Args:
        target: The target being analyzed
        ctx: The aspect context
        
    Returns:
        PackageInfo struct with package info or None if not a Maven artifact
    """
    label_str = str(ctx.label)
    
    # Determine scope based on attribute name where dependency is referenced
    scope = "compile"  # Default scope
    
    # Check if this is a Maven artifact (from rules_jvm_external)
    if JavaInfo in target:
        # Try to extract Maven coordinates from tags
        tags = getattr(ctx.rule.attr, "tags", [])
        for tag in tags:
            if tag.startswith("maven_coordinates="):
                coords = tag[len("maven_coordinates="):]
                # Parse group:artifact:version[:packaging[:classifier]]
                parts = coords.split(":")
                if len(parts) >= 3:
                    group = parts[0]
                    name = parts[1]
                    version = parts[2]
                    
                    # Build PURL with proper encoding
                    # Format: pkg:maven/group/name@version
                    purl = "pkg:maven/{}/{}@{}".format(
                        group.replace(".", "/"),
                        name,
                        version,
                    )
                    
                    return PackageInfo(
                        name = name,
                        group = group,
                        version = version,
                        purl = purl,
                        type = "maven",
                        label = label_str,
                        sha256 = "",  # Will be populated from maven_install.json
                        is_direct = False,  # Will be determined by parent
                        scope = scope,
                    )
    
    # For targets from @maven repository, try to parse the label
    # These are from rules_jvm_external maven_install
    if label_str.startswith("@maven//"):
        # Extract artifact name from label like @maven//:com_google_guava_guava
        artifact_name = label_str.replace("@maven//:", "")
        
        # Convert underscores back to dots and colons for Maven coordinates
        # Format: group_artifact becomes group:artifact
        # This is a heuristic and requires maven_install.json for full accuracy
        parts = artifact_name.split("_")
        
        # Try to reconstruct group:artifact from the label
        # This is imperfect but works for common patterns
        if len(parts) >= 2:
            # Common pattern: com_google_guava_guava -> com.google.guava:guava
            # We'll need to parse maven_install.json for exact coordinates
            return PackageInfo(
                name = artifact_name,
                group = "",  # Populated from maven_install.json
                version = "",  # Populated from maven_install.json
                purl = "",  # Constructed after coordinates are resolved
                type = "maven",
                label = label_str,
                sha256 = "",
                is_direct = False,
                scope = scope,
            )
    
    return None

def _sbom_aspect_impl(target, ctx):
    """Aspect implementation to collect dependency information.
    
    This aspect traverses the build graph and collects information about
    all dependencies for SBOM generation.
    
    Args:
        target: The target being analyzed
        ctx: The aspect context
        
    Returns:
        SbomInfo provider with package information
    """
    # Extract package information from this target
    pkg_info = _extract_maven_coordinates(target, ctx)
    
    # Build list of direct packages
    direct_packages = []
    if pkg_info:
        direct_packages.append(pkg_info)
    
    # Collect transitive dependencies from deps, runtime_deps, and exports
    transitive_packages = []
    for attr in ["deps", "runtime_deps", "exports"]:
        for dep in getattr(ctx.rule.attr, attr, []):
            if SbomInfo in dep:
                transitive_packages.append(dep[SbomInfo].packages)
    
    # Combine direct and transitive into a depset
    # Note: PackageInfo is a provider (immutable struct), so it's safe to use in depsets
    all_packages = depset(
        direct = direct_packages,
        transitive = transitive_packages,
    )
    
    return [SbomInfo(
        packages = all_packages,
        target_label = str(ctx.label),
    )]

sbom_aspect = aspect(
    implementation = _sbom_aspect_impl,
    attr_aspects = ["deps", "runtime_deps", "exports"],
    required_providers = [],
    provides = [SbomInfo],
    doc = """Aspect to collect dependency information for SBOM generation.
    
    This aspect traverses the dependency graph and collects metadata about
    all dependencies, including:
    - Package coordinates (Maven group:artifact:version)
    - Package URLs (PURLs)
    - Type information
    
    The aspect collects both direct and transitive dependencies.
    
    Usage:
        bazel build //path/to:target \\
          --aspects=//tools/supplychain:aspects.bzl%sbom_aspect \\
          --output_groups=sbom_info
    """,
)

ClasspathInfo = provider(
    doc = "Runtime classpath information for reachability analysis",
    fields = {
        "jars": "Depset of JAR file paths",
        "target_label": "Label of the target",
    },
)

def _classpath_aspect_impl(target, ctx):
    """Aspect implementation to collect runtime classpath JARs.
    
    This aspect collects all JAR files from the runtime classpath of a Java target,
    which can be used for bytecode reachability analysis.
    
    Args:
        target: The target being analyzed
        ctx: The aspect context
        
    Returns:
        ClasspathInfo provider with JAR file paths
    """
    direct_jars = []
    
    # Extract JAR files from JavaInfo provider
    if JavaInfo in target:
        java_info = target[JavaInfo]
        
        # Get runtime classpath JARs
        if hasattr(java_info, "transitive_runtime_jars"):
            for jar in java_info.transitive_runtime_jars.to_list():
                direct_jars.append(jar.path)
        
        # Also get compile-time JARs if runtime not available
        if not direct_jars and hasattr(java_info, "transitive_compile_time_jars"):
            for jar in java_info.transitive_compile_time_jars.to_list():
                direct_jars.append(jar.path)
    
    # Collect transitive JAR paths from dependencies
    transitive_jars = []
    for attr in ["deps", "runtime_deps", "exports"]:
        for dep in getattr(ctx.rule.attr, attr, []):
            if ClasspathInfo in dep:
                transitive_jars.append(dep[ClasspathInfo].jars)
    
    # Combine into depset
    all_jars = depset(
        direct = direct_jars,
        transitive = transitive_jars,
    )
    
    return [ClasspathInfo(
        jars = all_jars,
        target_label = str(ctx.label),
    )]

classpath_aspect = aspect(
    implementation = _classpath_aspect_impl,
    attr_aspects = ["deps", "runtime_deps", "exports"],
    required_providers = [],
    provides = [ClasspathInfo],
    doc = """Aspect to collect runtime classpath JARs for reachability analysis.
    
    This aspect traverses the dependency graph and collects all JAR files from
    the runtime classpath. These JARs are used by the reachability analyzer to
    perform bytecode analysis.
    
    Usage:
        bazel build //path/to:target \\
          --aspects=//tools/supplychain:aspects.bzl%classpath_aspect \\
          --output_groups=classpath_info
    """,
)
