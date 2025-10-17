"""Bazel aspects for dependency discovery and SBOM generation."""

SbomInfo = provider(
    doc = "Information about dependencies for SBOM generation",
    fields = {
        "packages": "Depset of package information dicts",
        "target_label": "Label of the target",
    },
)

def _extract_maven_coordinates(target, ctx):
    """Extract Maven coordinates from a target if available.
    
    Args:
        target: The target being analyzed
        ctx: The aspect context
        
    Returns:
        Dict with package info or None
    """
    # Check if this is a Maven artifact (from rules_jvm_external)
    if JavaInfo in target:
        # Try to extract Maven coordinates from tags or labels
        tags = getattr(ctx.rule.attr, "tags", [])
        for tag in tags:
            if tag.startswith("maven_coordinates="):
                coords = tag[len("maven_coordinates="):]
                # Parse group:artifact:version
                parts = coords.split(":")
                if len(parts) >= 3:
                    return {
                        "name": parts[1],
                        "group": parts[0],
                        "version": parts[2],
                        "purl": "pkg:maven/{}/{}@{}".format(parts[0], parts[1], parts[2]),
                        "type": "maven",
                    }
    
    # For targets without Maven coordinates, extract from label
    label = str(ctx.label)
    if label.startswith("@maven//"):
        # Parse @maven//:group_artifact format
        parts = label.replace("@maven//:", "").replace("_", ".")
        # This is a simplified parser - real implementation would need maven_install.json
        return {
            "name": parts,
            "type": "maven",
            "label": label,
        }
    
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
    # Collect packages from this target
    direct_packages = []
    
    # Extract package information from this target
    pkg_info = _extract_maven_coordinates(target, ctx)
    if pkg_info:
        direct_packages.append(pkg_info)
    
    # Collect transitive dependencies from deps, runtime_deps, and exports
    transitive_packages = []
    for attr in ["deps", "runtime_deps", "exports"]:
        for dep in getattr(ctx.rule.attr, attr, []):
            if SbomInfo in dep:
                transitive_packages.append(dep[SbomInfo].packages)
    
    # Combine direct and transitive into a depset
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
