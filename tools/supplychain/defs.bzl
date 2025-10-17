"""Bazel rules and macros for supply chain security."""

load(":aspects.bzl", "SbomInfo", "sbom_aspect")

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
        packages = target[SbomInfo].packages.to_list()
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
