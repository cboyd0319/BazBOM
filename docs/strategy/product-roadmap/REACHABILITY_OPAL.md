# Reachability Analysis with OPAL

Decision: OPAL is the preferred backbone for bytecode reachability.

Objectives
- Determine whether vulnerable code paths are actually reachable from application entrypoints
- Provide method-level traces to aid triage and remediation
- Cache results and scope analysis to minimize overhead

Integration Plan
1) Build a small JVM tool (`bazbom-reachability.jar`) using OPAL
   - Inputs: list of entrypoints, runtime classpath, exclusion filters
   - Output: JSON with reachable packages/classes/methods, and component→GAV mapping

2) Entrypoints Resolution
   - Maven/Gradle: derive from application plugins or main classes per module
   - Bazel: derive from `java_binary`/`java_test` targets and classpaths

3) Shaded JAR Awareness
   - Use relocation maps from Maven/Gradle to re-map classes back to original GAVs
   - Fallback: fingerprint-based matching for nested/shaded classes

4) Performance Controls
   - Scope by target/module; limit depth; allow package filters
   - Persistent cache keyed by classpath hash and entrypoint signature

5) Output Semantics
   - Annotate findings with `reachable: true|false`, `entrypoints: [...]`, and `trace: [...]`
   - Include rationale in CLI: KEV/EPSS + reachability in “why fix this?”

6) Security
   - JVM tool packaged with checksums; no network access; runs locally

