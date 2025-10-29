package io.bazbom.gradle

import org.gradle.api.Plugin
import org.gradle.api.Project

/**
 * BazBOM Gradle Plugin for generating authoritative dependency graphs.
 * 
 * This plugin provides tasks to capture comprehensive dependency information
 * including configurations, variants, and Android-specific details.
 */
class BazBomPlugin implements Plugin<Project> {
    
    @Override
    void apply(Project project) {
        // Register the extension for configuration
        BazBomExtension extension = project.extensions.create('bazbom', BazBomExtension)
        
        // Register the graph generation task
        project.tasks.register('bazbomGraph', BazBomGraphTask) { task ->
            task.group = 'bazbom'
            task.description = 'Generate dependency graph JSON for BazBOM analysis'
            task.outputFile.set(project.layout.buildDirectory.file('bazbom-graph.json'))
        }
        
        // Register the SBOM generation task
        project.tasks.register('bazbomSbom', BazBomSbomTask) { task ->
            task.group = 'bazbom'
            task.description = 'Generate SBOM for the project'
            task.dependsOn('bazbomGraph')
        }
        
        // Register the findings task
        project.tasks.register('bazbomFindings', BazBomFindingsTask) { task ->
            task.group = 'bazbom'
            task.description = 'Generate security findings report'
            task.dependsOn('bazbomGraph')
        }
        
        project.logger.info("BazBOM Gradle Plugin applied to project: ${project.name}")
    }
}
