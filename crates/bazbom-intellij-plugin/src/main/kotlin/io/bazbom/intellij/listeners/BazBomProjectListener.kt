package io.bazbom.intellij.listeners

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.ProjectManagerListener
import io.bazbom.intellij.services.BazBomProjectService
import io.bazbom.intellij.settings.BazBomSettings
import io.bazbom.intellij.util.BazBomCliRunner

/**
 * Listener for project lifecycle events.
 */
class BazBomProjectListener : ProjectManagerListener {
    private val log = Logger.getInstance(BazBomProjectListener::class.java)
    
    override fun projectOpened(project: Project) {
        log.info("Project opened: ${project.name}")
        
        val settings = BazBomSettings.getInstance()
        if (settings.autoScanOnOpen) {
            log.info("Auto-scan on open enabled, starting scan")
            
            ProgressManager.getInstance().run(object : Task.Backgroundable(project, "BazBOM Initial Scan", false) {
                override fun run(indicator: ProgressIndicator) {
                    indicator.text = "Running initial BazBOM scan..."
                    indicator.isIndeterminate = false
                    
                    try {
                        val runner = BazBomCliRunner(project)
                        val result = runner.runScan(fast = true)
                        
                        if (result.success) {
                            log.info("Initial scan completed successfully")
                            val service = project.getService(BazBomProjectService::class.java)
                            service.loadLastResults()
                        } else {
                            log.warn("Initial scan failed: ${result.errorMessage}")
                        }
                    } catch (e: Exception) {
                        log.error("Exception during initial scan", e)
                    }
                }
            })
        }
    }
    
    override fun projectClosing(project: Project) {
        log.info("Project closing: ${project.name}")
        
        val service = project.getService(BazBomProjectService::class.java)
        service.clearResults()
    }
}
