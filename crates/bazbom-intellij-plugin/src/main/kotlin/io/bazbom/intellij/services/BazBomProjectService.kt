package io.bazbom.intellij.services

import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import io.bazbom.intellij.model.DependencyNode
import io.bazbom.intellij.util.SbomParser
import java.io.File

/**
 * Project-level service for caching BazBOM scan results.
 */
@Service(Service.Level.PROJECT)
class BazBomProjectService(private val project: Project) {
    private val log = Logger.getInstance(BazBomProjectService::class.java)
    
    // Cache for scan results
    private var lastScanResults: DependencyNode? = null
    private var lastScanTime: Long = 0
    
    init {
        log.info("BazBomProjectService initialized for project: ${project.name}")
    }
    
    fun updateScanResults(results: DependencyNode) {
        lastScanResults = results
        lastScanTime = System.currentTimeMillis()
        log.info("Updated scan results: ${results.children.size} scope groups")
    }
    
    fun getLastScanResults(): DependencyNode? = lastScanResults
    
    fun getLastScanTime(): Long = lastScanTime
    
    fun loadLastResults() {
        try {
            val sbomFile = File(project.basePath, "sbom.spdx.json")
            if (sbomFile.exists()) {
                val dependencyTree = SbomParser.parseSbom(sbomFile)
                if (dependencyTree != null) {
                    updateScanResults(dependencyTree)
                    log.info("Loaded scan results from ${sbomFile.absolutePath}")
                } else {
                    log.warn("Failed to parse SBOM file at ${sbomFile.absolutePath}")
                }
            } else {
                log.warn("SBOM file not found at ${sbomFile.absolutePath}")
            }
        } catch (e: Exception) {
            log.error("Failed to load scan results", e)
        }
    }
    
    fun clearResults() {
        lastScanResults = null
        lastScanTime = 0
        log.info("Cleared scan results")
    }
    
    fun clearCache() {
        clearResults()
    }
}
