# BazBOM JVM-Focused Priorities

**Date:** 2025-11-04  
**Status:** Corrective Action - Refocusing on Core Mission

---

## Mission Reaffirmation

> **BazBOM Mission:** World-class JVM SBOM, SCA, and dependency graph across Maven, Gradle, and Bazel.

**Primary Focus:** Java, Kotlin, Scala projects  
**Build Systems:** Maven, Gradle, Bazel  
**Audience:** Enterprise/AppSec engineers working with JVM ecosystems

---

## What We Removed

The `bazbom-ecosystems` crate with multi-language support (Node.js, Python, Go, Rust) has been removed as it does not align with BazBOM's core JVM mission. Phase 9 "Ecosystem Expansion" should be reinterpreted as:
- Container scanning for JVM applications (Docker/OCI with Java artifacts)
- NOT multi-language package manager support

---

## Top JVM-Focused Priorities

### P0: IDE Marketplace Publishing (Phase 4 - 5% Remaining)

**Goal:** Get BazBOM IDE plugins into VS Code and JetBrains marketplaces with JVM-specific features prominently displayed.

**Tasks:**
1. **Testing with Real JVM Projects**
   - Test IntelliJ plugin with large Maven monorepo
   - Test IntelliJ plugin with multi-module Gradle project  
   - Test IntelliJ plugin with Bazel JVM workspace
   - Test VS Code extension with same project types
   - Document any JVM-specific issues

2. **Demo Content Creation**
   - Record 2-minute video: "Finding Java vulnerabilities with BazBOM in IntelliJ"
   - Record 2-minute video: "One-click Maven dependency fixes in VS Code"
   - Screenshots showing Java vulnerability detection
   - Screenshots showing Kotlin dependency tree
   - GIF showing quick-fix action on vulnerable Java library

3. **Marketplace Preparation**
   - Update README to emphasize JVM focus
   - Add keywords: "Java", "Kotlin", "Scala", "Maven", "Gradle", "Bazel"
   - Create marketplace banner emphasizing JVM  
   - Write clear description highlighting JVM build system support
   - Package and test .vsix and .zip files

4. **Publication**
   - Submit to VS Code Marketplace
   - Submit to JetBrains Plugin Repository
   - Announce on relevant JVM/security communities

**Timeline:** 1-2 weeks

---

### P1: Bazel JVM Performance (Phase 8 - 10% Remaining)

**Goal:** Make BazBOM the fastest and most accurate SBOM tool for Bazel JVM projects.

**Tasks:**
1. **Query Optimization**
   - Benchmark current Bazel query performance on 10K+ target monorepo
   - Implement query result caching specific to Java rules
   - Optimize `java_library`, `java_binary`, `java_test` traversal
   - Add parallel query execution for independent subtrees

2. **JVM Rule Support Enhancement**
   - Improve detection of `kt_jvm_library` (Kotlin)
   - Add support for `scala_library`, `scala_binary`
   - Better handling of `java_proto_library`
   - Support for `android_library`, `android_binary` with JVM deps

3. **Memory Optimization**
   - Profile memory usage on large JVM dependency graphs
   - Implement streaming for maven_install.json parsing
   - Reduce in-memory graph representation
   - Add configurable memory limits

4. **Performance Benchmarks**
   - Create benchmark suite with 1K, 10K, 50K JVM targets
   - Measure scan time vs. other tools
   - Document performance improvements
   - Add CI performance regression tests

**Timeline:** 2-3 weeks

---

### P2: JVM-Specific Feature Enhancements

**Goal:** Improve accuracy and depth of JVM dependency analysis.

**Tasks:**
1. **Fat JAR Detection**
   - Enhance Maven Shade plugin detection
   - Better Gradle Shadow plugin analysis
   - Detect shaded dependencies within JARs
   - Generate accurate PURLs for shaded artifacts

2. **Multi-Module Project Support**
   - Improve Maven multi-module reactor handling
   - Better Gradle composite build support
   - Bazel workspace boundary detection
   - Cross-module dependency tracking

3. **JVM Version Detection**
   - Detect Java target version from build files
   - Warn about Java version mismatches
   - Track Kotlin language version
   - Scala 2.x vs 3.x detection

4. **Android-Specific Features**
   - Android library (AAR) dependency analysis
   - AndroidX vs support library detection
   - ProGuard/R8 configuration analysis
   - Android SDK version tracking

**Timeline:** 3-4 weeks

---

### P3: Documentation Clarity

**Goal:** Ensure all documentation clearly states JVM-only focus.

**Tasks:**
1. **Update README.md**
   - Add prominent "JVM-focused" badge
   - Remove any multi-language references
   - Update feature list to emphasize Maven/Gradle/Bazel
   - Add "Scope" section explicitly stating JVM-only

2. **Update ROADMAP.md**
   - Remove or clearly mark multi-language items as "out of scope"
   - Reinterpret Phase 9 as "JVM Container Scanning" not "Multi-Language"
   - Emphasize JVM-specific improvements throughout
   - Update completion percentages to reflect JVM mission only

3. **Create JVM Examples**
   - Example: Large Spring Boot Maven project
   - Example: Android Gradle multi-module app
   - Example: Bazel monorepo with Java/Kotlin/Scala
   - Example: Fat JAR with shaded dependencies

4. **Update Capabilities Reference**
   - Emphasize JVM build system integrations
   - Highlight Bazel aspect support for Java rules
   - Document Java-specific reachability analysis
   - Clarify scope excludes other languages

**Timeline:** 1 week

---

## Metrics for Success

### IDE Plugin Metrics
- [ ] 1000+ VS Code extension installs (first 3 months)
- [ ] 500+ IntelliJ plugin downloads (first 3 months)
- [ ] 4.5+ star rating on both marketplaces
- [ ] <5 second scan time for 100 JVM dependencies
- [ ] 95% fix success rate on Java/Kotlin/Scala projects

### Performance Metrics
- [ ] Bazel scan of 10K Java targets in <2 minutes
- [ ] Maven scan of 500 dependencies in <30 seconds
- [ ] Gradle scan of multi-module project in <45 seconds
- [ ] Memory usage <2GB for largest JVM projects

### Adoption Metrics
- [ ] 10+ enterprises using BazBOM for JVM projects
- [ ] 50+ GitHub stars on project
- [ ] Featured in Bazel community showcase
- [ ] Referenced in Java security tooling comparisons

---

## Out of Scope (NOT Aligned with Mission)

The following are explicitly **out of scope** for BazBOM:

- ❌ Node.js/npm package analysis
- ❌ Python/pip package analysis
- ❌ Go modules package analysis
- ❌ Rust/Cargo package analysis
- ❌ Generic multi-language support
- ❌ Non-JVM polyglot projects
- ❌ JavaScript/TypeScript dependency trees
- ❌ C/C++ dependency management

**Exception:** Container scanning is in scope ONLY for detecting Java artifacts (JARs) within containers, not for multi-language container dependency analysis.

---

## Revised Phase Interpretations

### Phase 4: Developer Experience (IDE Integration)
**Scope:** JVM-focused IDE plugins with Java/Kotlin/Scala vulnerability detection
**Status:** 95% complete

### Phase 7: Threat Intelligence
**Scope:** Malicious JVM package detection, Maven Central typosquatting
**Status:** 95% complete

### Phase 8: Scale & Performance
**Scope:** Optimize for large Bazel JVM monorepos, Maven/Gradle performance
**Status:** 90% complete

### Phase 9: JVM Container Scanning (REINTERPRETED)
**Scope:** Scan Docker/OCI images for Java artifacts, not multi-language
**Status:** 60% complete (containers only)

### Phase 10: AI Intelligence
**Scope:** ML-based Java vulnerability prioritization (future)
**Status:** Planned

### Phase 11: Enterprise Distribution
**Scope:** Windows support, K8s operators for JVM scanning, air-gapped JVM repos
**Status:** Planned

---

## Implementation Focus

**This Week:**
1. Clean up any multi-language references in docs
2. Test IDE plugins with real Java/Kotlin projects
3. Create demo videos emphasizing JVM support
4. Start marketplace listing preparation

**Next 2 Weeks:**
1. Submit to VS Code Marketplace
2. Submit to JetBrains Marketplace
3. Begin Bazel performance profiling
4. Update all documentation for JVM clarity

**Next Month:**
1. Complete Bazel JVM optimization
2. Add Android-specific features
3. Improve fat JAR detection
4. Performance benchmarks published

---

## Questions & Clarifications

**Q: Can BazBOM scan containers?**  
A: Yes, but only to extract Java artifacts (JARs) from container layers, not for multi-language analysis.

**Q: Will BazBOM support other languages in the future?**  
A: No, BazBOM is focused on being the best JVM SBOM tool. Multi-language is out of scope.

**Q: What about JavaScript in JVM projects (e.g., frontend)?**  
A: Use a JavaScript-specific tool for that. BazBOM focuses on the JVM backend dependencies.

**Q: Can BazBOM analyze Kotlin Multiplatform?**  
A: Only the JVM targets, not JS, Native, or WASM targets.

---

## Conclusion

BazBOM's value proposition is **world-class JVM SBOM generation** with deep Maven, Gradle, and Bazel integration. By staying focused on this mission, we can deliver the best tool in this space rather than a mediocre multi-language alternative.

**Next Actions:**
1. ✅ Remove multi-language ecosystem crate
2. Test IDE plugins with JVM projects
3. Create marketplace demos
4. Update documentation for clarity
5. Begin Bazel JVM optimization

---

**Document Owner:** @cboyd0319  
**Last Updated:** 2025-11-04  
**Status:** Active - Corrective Action in Progress
