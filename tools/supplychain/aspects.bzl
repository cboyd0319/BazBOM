"""Bazel aspects for dependency discovery and SBOM generation."""

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
    
    Args:
        target: The target being analyzed
        ctx: The aspect context
        
    Returns:
        PackageInfo struct with package info or None
    """
    label_str = str(ctx.label)
    
    # Check if this is a Maven artifact (from rules_jvm_external)
    if JavaInfo in target:
        # Try to extract Maven coordinates from tags
        tags = getattr(ctx.rule.attr, "tags", [])
        for tag in tags:
            if tag.startswith("maven_coordinates="):
                coords = tag[len("maven_coordinates="):]
                # Parse group:artifact:version
                parts = coords.split(":")
                if len(parts) >= 3:
                    group = parts[0]
                    name = parts[1]
                    version = parts[2]
                    return PackageInfo(
                        name = name,
                        group = group,
                        version = version,
                        purl = "pkg:maven/{}/{}@{}".format(group, name, version),
                        type = "maven",
                        label = label_str,
                        sha256 = "",
                        is_direct = False,  # Will be determined by parent
                    )
    
    # For targets from @maven repository, try to parse the label
    if label_str.startswith("@maven//"):
        # Extract artifact name from label like @maven//:com_google_guava_guava
        artifact_name = label_str.replace("@maven//:", "")
        
        # The artifact name uses underscores, we need to map this back to Maven coordinates
        # This is a heuristic - ideally we'd read maven_install.json
        # For now, just record the label for provenance
        return PackageInfo(
            name = artifact_name,
            group = "",  # Unknown without maven_install.json
            version = "",  # Unknown without maven_install.json
            purl = "",  # Cannot construct without full coordinates
            type = "maven",
            label = label_str,
            sha256 = "",
            is_direct = False,
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
