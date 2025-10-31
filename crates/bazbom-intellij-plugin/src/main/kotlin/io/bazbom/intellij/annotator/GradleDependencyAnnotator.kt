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
 * Annotator for Gradle build.gradle and build.gradle.kts files.
 * Highlights vulnerable dependencies with inline warnings.
 */
class GradleDependencyAnnotator : Annotator {
    private val log = Logger.getInstance(GradleDependencyAnnotator::class.java)
    
    // Pattern to match: implementation 'group:artifact:version'
    // or implementation("group:artifact:version")
    private val dependencyPattern = Regex(
        """['"]([^:'"]+):([^:'"]+):([^'"]+)['"]"""
    )
    
    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        // Only process Gradle files
        val fileName = element.containingFile.name
        if (!fileName.endsWith("build.gradle") && !fileName.endsWith("build.gradle.kts")) {
            return
        }
        
        // Check if this element contains a dependency declaration
        val text = element.text
        val match = dependencyPattern.find(text) ?: return
        
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
