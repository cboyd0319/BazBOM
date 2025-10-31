package io.bazbom.intellij.annotator

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.diagnostic.Logger
import com.intellij.psi.PsiElement
import com.intellij.psi.xml.XmlTag
import io.bazbom.intellij.services.BazBomProjectService
import io.bazbom.intellij.util.SbomParser
import java.io.File

/**
 * Annotator for Maven pom.xml files.
 * Highlights vulnerable dependencies with inline warnings.
 */
class MavenDependencyAnnotator : Annotator {
    private val log = Logger.getInstance(MavenDependencyAnnotator::class.java)
    
    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        // Only process XML tags
        if (element !is XmlTag) return
        
        // Only process dependency tags in pom.xml
        if (element.name != "dependency") return
        if (!element.containingFile.name.equals("pom.xml", ignoreCase = true)) return
        
        // Extract dependency coordinates
        val groupId = element.findFirstSubTag("groupId")?.value?.text
        val artifactId = element.findFirstSubTag("artifactId")?.value?.text
        val version = element.findFirstSubTag("version")?.value?.text
        
        if (groupId == null || artifactId == null || version == null) {
            return
        }
        
        // Build PURL
        val purl = "pkg:maven/$groupId/$artifactId@$version"
        
        // Get vulnerabilities from scan results
        val vulnerabilities = findVulnerabilitiesForPurl(element, purl)
        
        if (vulnerabilities.isEmpty()) {
            return
        }
        
        // Determine severity
        val hasCritical = vulnerabilities.any { it.contains("CRITICAL") }
        val hasHigh = vulnerabilities.any { it.contains("HIGH") }
        
        val severity = when {
            hasCritical -> HighlightSeverity.ERROR
            hasHigh -> HighlightSeverity.WARNING
            else -> HighlightSeverity.WEAK_WARNING
        }
        
        // Get version tag for annotation range
        val versionTag = element.findFirstSubTag("version")
        if (versionTag != null) {
            val message = buildAnnotationMessage(artifactId, vulnerabilities)
            
            holder.newAnnotation(severity, message)
                .range(versionTag.textRange)
                .create()
        }
    }
    
    private fun findVulnerabilitiesForPurl(element: PsiElement, purl: String): List<String> {
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
    
    private fun findVulnerabilitiesInTree(node: io.bazbom.intellij.model.DependencyNode, targetPurl: String): List<String> {
        val vulns = mutableListOf<String>()
        
        if (node.purl == targetPurl) {
            node.vulnerabilities.forEach { vuln ->
                val severity = vuln.severity ?: "UNKNOWN"
                val kevWarning = if (vuln.cisaKev) " (CISA KEV)" else ""
                val reachableWarning = if (vuln.reachable) " (Reachable)" else ""
                val summary = vuln.summary ?: "No description"
                vulns.add("${vuln.id} ($severity)$kevWarning$reachableWarning: $summary")
            }
        }
        
        node.children.forEach { child ->
            vulns.addAll(findVulnerabilitiesInTree(child, targetPurl))
        }
        
        return vulns
    }
    
    private fun buildAnnotationMessage(artifactId: String, vulnerabilities: List<String>): String {
        return if (vulnerabilities.size == 1) {
            "Vulnerability in $artifactId: ${vulnerabilities[0]}"
        } else {
            "Multiple vulnerabilities in $artifactId:\n" + vulnerabilities.joinToString("\n") { "  - $it" }
        }
    }
}
