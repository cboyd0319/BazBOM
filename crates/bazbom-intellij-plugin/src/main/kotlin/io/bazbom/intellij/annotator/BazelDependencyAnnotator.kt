package io.bazbom.intellij.annotator

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.diagnostic.Logger
import com.intellij.psi.PsiElement
import io.bazbom.intellij.quickfix.UpgradeDependencyQuickFix
import io.bazbom.intellij.util.SbomParser
import java.io.File

/**
 * Annotator for Bazel BUILD, BUILD.bazel, WORKSPACE, and MODULE.bazel files.
 * Highlights vulnerable Maven dependencies with inline warnings.
 */
class BazelDependencyAnnotator : Annotator {
    private val log = Logger.getInstance(BazelDependencyAnnotator::class.java)
    
    // Pattern to match Maven coordinates in Bazel files
    // Examples:
    // - "group:artifact:version"
    // - maven.artifact(group = "group", artifact = "artifact", version = "version")
    private val coordinatePattern = Regex(
        """['"]([^:'"]+):([^:'"]+):([^'"]+)['"]"""
    )
    
    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        // Only process Bazel files
        val fileName = element.containingFile.name
        if (!isBazelFile(fileName)) {
            return
        }
        
        // Check if this element contains a Maven coordinate
        val text = element.text
        val match = coordinatePattern.find(text) ?: return
        
        val (groupId, artifactId, version) = match.destructured
        
        // Build PURL
        val purl = "pkg:maven/$groupId/$artifactId@$version"
        
        // Get vulnerabilities from scan results
        val vulnerabilities = findVulnerabilitiesForPurl(element, purl)
        
        if (vulnerabilities.isEmpty()) {
            return
        }
        
        // Determine severity
        val hasCritical = vulnerabilities.any { it.severity?.contains("CRITICAL") == true }
        val hasHigh = vulnerabilities.any { it.severity?.contains("HIGH") == true }
        
        val severity = when {
            hasCritical -> HighlightSeverity.ERROR
            hasHigh -> HighlightSeverity.WARNING
            else -> HighlightSeverity.WEAK_WARNING
        }
        
        val message = buildAnnotationMessage(artifactId, vulnerabilities)
        
        // Extract fixed version from first vulnerability
        val fixedVersion = vulnerabilities.firstOrNull()?.fixedVersion
        
        val annotation = holder.newAnnotation(severity, message)
            .range(element.textRange)
        
        // Add quick fix if a fixed version is available
        if (fixedVersion != null) {
            annotation.withFix(
                UpgradeDependencyQuickFix(groupId, artifactId, version, fixedVersion)
            )
        }
        
        annotation.create()
    }
    
    private fun isBazelFile(fileName: String): Boolean {
        return fileName == "BUILD" ||
               fileName == "BUILD.bazel" ||
               fileName == "WORKSPACE" ||
               fileName == "WORKSPACE.bazel" ||
               fileName == "MODULE.bazel"
    }
    
    private fun findVulnerabilitiesForPurl(element: PsiElement, purl: String): List<VulnerabilityInfo> {
        val project = element.project
        val projectPath = project.basePath ?: return emptyList()
        
        // Try to find SBOM file
        val outputDir = File(projectPath, ".bazbom/scan-output")
        if (!outputDir.exists()) {
            return emptyList()
        }
        
        val sbomFile = SbomParser.findSbomFile(outputDir)
        if (sbomFile == null || !sbomFile.exists()) {
            return emptyList()
        }
        
        // Parse SBOM and find vulnerabilities for this PURL
        val dependencyTree = SbomParser.parseSbom(sbomFile) ?: return emptyList()
        
        return findVulnerabilitiesInTree(dependencyTree, purl)
    }
    
    private fun findVulnerabilitiesInTree(
        node: io.bazbom.intellij.model.DependencyNode,
        targetPurl: String
    ): List<VulnerabilityInfo> {
        val vulns = mutableListOf<VulnerabilityInfo>()
        
        if (node.purl == targetPurl) {
            node.vulnerabilities.forEach { vuln ->
                vulns.add(
                    VulnerabilityInfo(
                        id = vuln.id,
                        severity = vuln.severity,
                        summary = vuln.summary,
                        cisaKev = vuln.cisaKev,
                        reachable = vuln.reachable,
                        fixedVersion = vuln.fixedVersion
                    )
                )
            }
        }
        
        node.children.forEach { child ->
            vulns.addAll(findVulnerabilitiesInTree(child, targetPurl))
        }
        
        return vulns
    }
    
    private fun buildAnnotationMessage(artifactId: String, vulnerabilities: List<VulnerabilityInfo>): String {
        return if (vulnerabilities.size == 1) {
            val vuln = vulnerabilities[0]
            val kevWarning = if (vuln.cisaKev) " (CISA KEV)" else ""
            val reachableWarning = if (vuln.reachable) " (Reachable)" else ""
            val summary = vuln.summary ?: "No description"
            "Vulnerability in $artifactId: ${vuln.id} (${vuln.severity})$kevWarning$reachableWarning: $summary"
        } else {
            "Multiple vulnerabilities in $artifactId:\n" + vulnerabilities.joinToString("\n") { vuln ->
                val kevWarning = if (vuln.cisaKev) " (CISA KEV)" else ""
                val reachableWarning = if (vuln.reachable) " (Reachable)" else ""
                "  - ${vuln.id} (${vuln.severity})$kevWarning$reachableWarning"
            }
        }
    }
    
    private data class VulnerabilityInfo(
        val id: String,
        val severity: String?,
        val summary: String?,
        val cisaKev: Boolean,
        val reachable: Boolean,
        val fixedVersion: String?
    )
}
