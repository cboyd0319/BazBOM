package io.bazbom.intellij.util

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import java.io.File

/**
 * Utility class for running BazBOM CLI commands.
 */
class BazBomCliRunner(private val project: Project) {
    private val log = Logger.getInstance(BazBomCliRunner::class.java)
    
    data class ScanResult(
        val success: Boolean,
        val outputDir: File? = null,
        val errorMessage: String? = null
    )
    
    /**
     * Run BazBOM scan on the project.
     * 
     * @param fast If true, uses --fast mode (skips reachability analysis)
     * @return ScanResult with success status and output directory
     */
    fun runScan(fast: Boolean = true): ScanResult {
        val projectPath = project.basePath ?: return ScanResult(
            success = false,
            errorMessage = "Project path not found"
        )
        
        // Create output directory in project
        val outputDir = File(projectPath, ".bazbom/scan-output")
        outputDir.mkdirs()
        
        // Build command
        val command = mutableListOf("bazbom", "scan")
        if (fast) {
            command.add("--fast")
        }
        command.addAll(listOf("--out-dir", outputDir.absolutePath, projectPath))
        
        log.info("Running command: ${command.joinToString(" ")}")
        
        return try {
            val process = ProcessBuilder(command)
                .directory(File(projectPath))
                .redirectErrorStream(true)
                .start()
            
            val output = process.inputStream.bufferedReader().readText()
            val exitCode = process.waitFor()
            
            if (exitCode == 0) {
                log.info("BazBOM scan completed successfully")
                ScanResult(success = true, outputDir = outputDir)
            } else {
                log.error("BazBOM scan failed: $output")
                ScanResult(success = false, errorMessage = output)
            }
        } catch (e: Exception) {
            log.error("Exception running BazBOM scan", e)
            ScanResult(success = false, errorMessage = e.message)
        }
    }
}
