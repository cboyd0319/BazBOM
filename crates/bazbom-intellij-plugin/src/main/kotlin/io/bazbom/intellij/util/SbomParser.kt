package io.bazbom.intellij.util

import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import com.intellij.openapi.diagnostic.Logger
import io.bazbom.intellij.model.DependencyNode
import io.bazbom.intellij.model.SbomDocument
import io.bazbom.intellij.model.Vulnerability
import java.io.File

/**
 * Utility for parsing BazBOM SBOM JSON output.
 */
object SbomParser {
    private val log = Logger.getInstance(SbomParser::class.java)
    private val mapper = jacksonObjectMapper()
    
    /**
     * Parse SBOM JSON file and convert to dependency tree structure.
     */
    fun parseSbom(sbomFile: File): DependencyNode? {
        if (!sbomFile.exists()) {
            log.warn("SBOM file not found: ${sbomFile.absolutePath}")
            return null
        }
        
        return try {
            val sbom: SbomDocument = mapper.readValue(sbomFile)
            buildDependencyTree(sbom)
        } catch (e: Exception) {
            log.error("Failed to parse SBOM file: ${sbomFile.absolutePath}", e)
            null
        }
    }
    
    /**
     * Build dependency tree from SBOM document.
     */
    private fun buildDependencyTree(sbom: SbomDocument): DependencyNode {
        // Create vulnerability map for quick lookup
        val vulnMap = mutableMapOf<String, MutableList<Vulnerability>>()
        sbom.vulnerabilities.forEach { vuln ->
            vuln.affects.forEach { affected ->
                vulnMap.getOrPut(affected.ref) { mutableListOf() }.add(vuln)
            }
        }
        
        // Create root node
        val root = DependencyNode(
            name = "Project Dependencies",
            version = "",
            purl = "",
            scope = "all",
            vulnerabilities = emptyList()
        )
        
        // Group packages by scope
        val scopeMap = mutableMapOf<String, MutableList<DependencyNode>>()
        
        sbom.packages.forEach { pkg ->
            val purl = pkg.externalRefs
                .firstOrNull { it.referenceType == "purl" }
                ?.referenceLocator ?: ""
            
            val scope = pkg.scope ?: "compile"
            val vulns = vulnMap[pkg.spdxId] ?: emptyList()
            
            val node = DependencyNode(
                name = pkg.name,
                version = pkg.version,
                purl = purl,
                scope = scope,
                vulnerabilities = vulns
            )
            
            scopeMap.getOrPut(scope) { mutableListOf() }.add(node)
        }
        
        // Add scope nodes to root
        scopeMap.forEach { (scope, nodes) ->
            val scopeNode = DependencyNode(
                name = scope.uppercase(),
                version = "",
                purl = "",
                scope = scope,
                vulnerabilities = emptyList()
            )
            nodes.forEach { scopeNode.children.add(it) }
            root.children.add(scopeNode)
        }
        
        return root
    }
    
    /**
     * Find SBOM file in project output directory.
     */
    fun findSbomFile(outputDir: File): File? {
        // Look for sbom.spdx.json or sbom.json
        val possibleNames = listOf("sbom.spdx.json", "sbom.json", "bom.json")
        
        for (name in possibleNames) {
            val file = File(outputDir, name)
            if (file.exists()) {
                return file
            }
        }
        
        // Search in subdirectories
        outputDir.listFiles()?.forEach { file ->
            if (file.isFile && file.name.endsWith(".json")) {
                return file
            }
        }
        
        return null
    }
}
