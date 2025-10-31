package io.bazbom.intellij.services

import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project

/**
 * Project-level service for caching BazBOM scan results.
 */
@Service(Service.Level.PROJECT)
class BazBomProjectService(private val project: Project) {
    private val log = Logger.getInstance(BazBomProjectService::class.java)
    
    // Cache for scan results
    private var lastScanResults: Map<String, Any>? = null
    private var lastScanTime: Long = 0
    
    init {
        log.info("BazBomProjectService initialized for project: ${project.name}")
    }
    
    fun updateScanResults(results: Map<String, Any>) {
        lastScanResults = results
        lastScanTime = System.currentTimeMillis()
        log.info("Updated scan results")
    }
    
    fun getLastScanResults(): Map<String, Any>? = lastScanResults
    
    fun getLastScanTime(): Long = lastScanTime
    
    fun clearCache() {
        lastScanResults = null
        lastScanTime = 0
        log.info("Cleared scan results cache")
    }
}
