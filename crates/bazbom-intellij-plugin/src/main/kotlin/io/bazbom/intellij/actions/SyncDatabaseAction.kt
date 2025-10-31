package io.bazbom.intellij.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.intellij.openapi.ui.Messages

/**
 * Action to sync the BazBOM advisory database.
 */
class SyncDatabaseAction : AnAction() {
    private val log = Logger.getInstance(SyncDatabaseAction::class.java)

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        
        ProgressManager.getInstance().run(object : Task.Backgroundable(project, "Syncing BazBOM Advisory Database", false) {
            override fun run(indicator: ProgressIndicator) {
                indicator.text = "Downloading advisory data..."
                indicator.isIndeterminate = true
                
                try {
                    val process = ProcessBuilder("bazbom", "db", "sync")
                        .redirectErrorStream(true)
                        .start()
                    
                    val exitCode = process.waitFor()
                    
                    if (exitCode == 0) {
                        log.info("Advisory database sync completed")
                        Messages.showInfoMessage(
                            project,
                            "Advisory database synchronized successfully",
                            "BazBOM"
                        )
                    } else {
                        log.error("Advisory database sync failed with exit code: $exitCode")
                        Messages.showErrorDialog(
                            project,
                            "Failed to sync advisory database. Please check logs.",
                            "BazBOM Error"
                        )
                    }
                } catch (e: Exception) {
                    log.error("Exception during database sync", e)
                    Messages.showErrorDialog(
                        project,
                        "Exception during database sync: ${e.message}",
                        "BazBOM Error"
                    )
                }
            }
        })
    }
}
