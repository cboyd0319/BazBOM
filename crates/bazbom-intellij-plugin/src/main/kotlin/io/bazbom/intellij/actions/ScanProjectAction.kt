package io.bazbom.intellij.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import io.bazbom.intellij.util.BazBomCliRunner

/**
 * Action to manually trigger a BazBOM scan on the current project.
 */
class ScanProjectAction : AnAction() {
    private val log = Logger.getInstance(ScanProjectAction::class.java)

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        
        log.info("Starting BazBOM scan for project: ${project.name}")
        
        ProgressManager.getInstance().run(object : Task.Backgroundable(project, "Scanning with BazBOM", true) {
            override fun run(indicator: ProgressIndicator) {
                indicator.text = "Running BazBOM scan..."
                indicator.isIndeterminate = false
                
                try {
                    val runner = BazBomCliRunner(project)
                    val result = runner.runScan(fast = false)
                    
                    if (result.success) {
                        log.info("BazBOM scan completed successfully")
                        
                        val service = project.getService(io.bazbom.intellij.services.BazBomProjectService::class.java)
                        service.loadLastResults()
                        
                        com.intellij.notification.Notifications.Bus.notify(
                            com.intellij.notification.Notification(
                                "BazBOM",
                                "Scan Complete",
                                "BazBOM scan completed successfully",
                                com.intellij.notification.NotificationType.INFORMATION
                            ),
                            project
                        )
                    } else {
                        log.error("BazBOM scan failed: ${result.errorMessage}")
                        
                        com.intellij.notification.Notifications.Bus.notify(
                            com.intellij.notification.Notification(
                                "BazBOM",
                                "Scan Failed",
                                "BazBOM scan failed: ${result.errorMessage}",
                                com.intellij.notification.NotificationType.ERROR
                            ),
                            project
                        )
                    }
                } catch (e: Exception) {
                    log.error("Exception during BazBOM scan", e)
                    
                    com.intellij.notification.Notifications.Bus.notify(
                        com.intellij.notification.Notification(
                            "BazBOM",
                            "Scan Error",
                            "Exception during scan: ${e.message}",
                            com.intellij.notification.NotificationType.ERROR
                        ),
                        project
                    )
                }
            }
        })
    }

    override fun update(e: AnActionEvent) {
        val project = e.project
        e.presentation.isEnabled = project != null
    }
}
