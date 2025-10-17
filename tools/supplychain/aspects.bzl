"""Bazel aspects for dependency discovery and SBOM generation."""

SbomInfo = provider(
    doc = "Information about dependencies for SBOM generation",
    fields = {
        "packages": "List of package information",
        "transitive_packages": "Depset of transitive package information",
    },
)

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
    packages = []
    
    # TODO: Extract package information from target
    # This would include:
    # - Package coordinates (group:artifact:version)
    # - License information
    # - Checksums
    # - Source URLs
    
    # Collect transitive dependencies
    transitive_packages = []
    for dep in getattr(ctx.rule.attr, "deps", []):
        if SbomInfo in dep:
            transitive_packages.append(dep[SbomInfo].transitive_packages)
    
    # Combine direct and transitive
    all_packages = depset(
        direct = packages,
        transitive = transitive_packages,
    )
    
    return [SbomInfo(
        packages = packages,
        transitive_packages = all_packages,
    )]

sbom_aspect = aspect(
    implementation = _sbom_aspect_impl,
    attr_aspects = ["deps", "runtime_deps", "exports"],
    doc = """Aspect to collect dependency information for SBOM generation.
    
    This aspect traverses the dependency graph and collects metadata about
    all dependencies, including:
    - Package coordinates
    - License information  
    - Version information
    - Checksums and signatures
    
    Usage:
        bazel build //path/to:target --aspects=//tools/supplychain:aspects.bzl%sbom_aspect
    """,
)
