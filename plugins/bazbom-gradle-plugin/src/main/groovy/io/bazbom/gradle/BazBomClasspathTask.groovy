package io.bazbom.gradle

import org.gradle.api.DefaultTask
import org.gradle.api.artifacts.Configuration
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.TaskAction

/**
 * Gradle task to extract runtime classpath for reachability analysis.
 * Outputs a colon-separated (or semicolon on Windows) list of JAR paths.
 */
abstract class BazBomClasspathTask extends DefaultTask {
    
    @OutputFile
    abstract RegularFileProperty getOutputFile()
    
    @TaskAction
    void extractClasspath() {
        def classpathEntries = []
        
        // Get the runtime classpath configuration
        def runtimeConfig = project.configurations.findByName('runtimeClasspath')
        if (runtimeConfig == null) {
            logger.warn("BazBOM: runtimeClasspath configuration not found")
            runtimeConfig = project.configurations.findByName('runtime')
        }
        
        if (runtimeConfig != null && runtimeConfig.canBeResolved) {
            try {
                runtimeConfig.resolvedConfiguration.resolvedArtifacts.each { artifact ->
                    def file = artifact.file
                    if (file.exists() && file.name.endsWith('.jar')) {
                        classpathEntries.add(file.absolutePath)
                    }
                }
                logger.info("BazBOM: Found ${classpathEntries.size()} JAR files in runtime classpath")
            } catch (Exception e) {
                logger.warn("BazBOM: Could not resolve runtime classpath: ${e.message}")
            }
        }
        
        // Add the project's own compiled classes
        def classesDir = project.layout.buildDirectory.dir('classes/java/main').get().asFile
        if (classesDir.exists()) {
            classpathEntries.add(0, classesDir.absolutePath)
        }
        
        // Write classpath to file
        def outputFileObj = outputFile.get().asFile
        outputFileObj.parentFile.mkdirs()
        
        // Use platform-specific path separator
        def separator = System.getProperty('os.name').toLowerCase().contains('windows') ? ';' : ':'
        outputFileObj.text = classpathEntries.join(separator)
        
        logger.lifecycle("BazBOM: Extracted classpath with ${classpathEntries.size()} entries")
        logger.lifecycle("BazBOM: Classpath written to: ${outputFileObj.absolutePath}")
    }
}
