"""Bazel rules and macros for supply chain security."""

load("@bazel_skylib//lib:json.bzl", "json")
load(":aspects.bzl", "SbomInfo", "sbom_aspect", "ClasspathInfo", "classpath_aspect")

def _sbom_impl(ctx):
    """Implementation of the sbom rule.
    
    This rule applies the sbom_aspect to a target and generates an SBOM.
    
    Args:
        ctx: Rule context
        
    Returns:
        DefaultInfo with the generated SBOM file
    """
    target = ctx.attr.target
    
    # Get dependency information from the aspect
    if SbomInfo in target:
        packages_depset = target[SbomInfo].packages
        packages_list = packages_depset.to_list()
        
        # Convert PackageInfo structs to dicts for JSON serialization
        packages = []
        for pkg in packages_list:
            pkg_dict = {
                "name": pkg.name,
                "group": pkg.group,
                "version": pkg.version,
                "purl": pkg.purl,
                "type": pkg.type,
                "label": pkg.label,
                "sha256": pkg.sha256,
                "is_direct": pkg.is_direct,
            }
            packages.append(pkg_dict)
    else:
        packages = []
    
    # Create a JSON file with package information
    deps_json = ctx.actions.declare_file(ctx.label.name + "_deps.json")
    ctx.actions.write(
        output = deps_json,
        content = json.encode_indent({"packages": packages}, indent = "  "),
    )
    
    # Generate SPDX SBOM using the Python script
    sbom_file = ctx.actions.declare_file(ctx.label.name + ".spdx.json")
    
    ctx.actions.run(
        inputs = [deps_json],
        outputs = [sbom_file],
        executable = ctx.executable._write_sbom_tool,
        arguments = [
            "--input", deps_json.path,
            "--output", sbom_file.path,
            "--name", ctx.attr.target.label.name,
        ],
        mnemonic = "GenerateSBOM",
        progress_message = "Generating SBOM for %s" % ctx.label.name,
    )
    
    return [DefaultInfo(files = depset([sbom_file]))]

sbom = rule(
    implementation = _sbom_impl,
    attrs = {
        "target": attr.label(
            aspects = [sbom_aspect],
            doc = "The target to generate an SBOM for",
            mandatory = True,
        ),
        "_write_sbom_tool": attr.label(
            default = Label("//tools/supplychain:write_sbom"),
            executable = True,
            cfg = "exec",
        ),
    },
    doc = """Generate an SPDX SBOM for a target.
    
    This rule uses a Bazel aspect to traverse the dependency graph
    and generate a Software Bill of Materials in SPDX 2.3 format.
    
    Example:
        sbom(
            name = "app_sbom",
            target = "//app:deployable",
        )
    """,
)

def sbom_for(name, target, **kwargs):
    """Macro to generate an SBOM for a single target.
    
    Args:
        name: Name of the SBOM target
        target: The target to generate SBOM for
        **kwargs: Additional arguments passed to the sbom rule
    """
    sbom(
        name = name,
        target = target,
        **kwargs
    )

def _extract_classpath_impl(ctx):
    """Implementation of the extract_classpath rule.
    
    This rule applies the classpath_aspect to a target and writes
    the runtime classpath to a file.
    
    Args:
        ctx: Rule context
        
    Returns:
        DefaultInfo with the generated classpath file
    """
    target = ctx.attr.target
    
    # Get classpath information from the aspect
    jars = []
    if ClasspathInfo in target:
        jars = target[ClasspathInfo].jars.to_list()
    
    # Write classpath to file (colon-separated)
    classpath_file = ctx.actions.declare_file(ctx.label.name + ".txt")
    ctx.actions.write(
        output = classpath_file,
        content = ":".join(jars),
    )
    
    return [DefaultInfo(files = depset([classpath_file]))]

extract_classpath = rule(
    implementation = _extract_classpath_impl,
    attrs = {
        "target": attr.label(
            aspects = [classpath_aspect],
            doc = "The target to extract classpath from",
            mandatory = True,
        ),
    },
    doc = """Extract runtime classpath for a Java target.
    
    This rule uses a Bazel aspect to collect all JAR files from the
    runtime classpath of a Java target. The output is a text file with
    colon-separated JAR paths suitable for reachability analysis.
    
    Example:
        extract_classpath(
            name = "app_classpath",
            target = "//app:main",
        )
    """,
)
