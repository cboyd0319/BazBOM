package io.bazbom.gradle

import org.gradle.api.DefaultTask
import org.gradle.api.tasks.TaskAction

/**
 * Task to generate SBOM from the dependency graph.
 */
abstract class BazBomSbomTask extends DefaultTask {
    
    @TaskAction
    void generateSbom() {
        logger.lifecycle("BazBOM: SBOM generation task (placeholder)")
        logger.lifecycle("BazBOM: This task will invoke the BazBOM CLI to generate SBOM from the graph")
    }
}
