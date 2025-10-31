package io.bazbom.intellij

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.StartupActivity

/**
 * Main entry point for BazBOM IntelliJ plugin.
 * Initializes plugin services and registers listeners.
 */
class BazBomPlugin : StartupActivity {
    private val log = Logger.getInstance(BazBomPlugin::class.java)

    override fun runActivity(project: Project) {
        log.info("BazBOM plugin initialized for project: ${project.name}")
        
        // Check if bazbom CLI is available
        val cliAvailable = checkBazBomCli()
        if (!cliAvailable) {
            log.warn("BazBOM CLI not found in PATH. Some features may not work.")
        }
    }

    private fun checkBazBomCli(): Boolean {
        return try {
            val process = ProcessBuilder("bazbom", "--version")
                .redirectErrorStream(true)
                .start()
            process.waitFor()
            process.exitValue() == 0
        } catch (e: Exception) {
            false
        }
    }
}
