package io.bazbom.intellij.listeners

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.ProjectManagerListener

/**
 * Listener for project lifecycle events.
 */
class BazBomProjectListener : ProjectManagerListener {
    private val log = Logger.getInstance(BazBomProjectListener::class.java)
    
    override fun projectOpened(project: Project) {
        log.info("Project opened: ${project.name}")
        // TODO: Check if auto-scan on open is enabled
    }
    
    override fun projectClosing(project: Project) {
        log.info("Project closing: ${project.name}")
        // TODO: Clean up resources
    }
}
