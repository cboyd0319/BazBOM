package io.bazbom.gradle

import com.google.gson.GsonBuilder
import org.gradle.api.DefaultTask
import org.gradle.api.artifacts.Configuration
import org.gradle.api.artifacts.ResolvedDependency
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.TaskAction

/**
 * Gradle task to generate a dependency graph JSON file.
 */
abstract class BazBomGraphTask extends DefaultTask {
    
    @OutputFile
    abstract RegularFileProperty getOutputFile()
    
    @TaskAction
    void generateGraph() {
        def graphData = [
            version: '1.0',
            generator: 'bazbom-gradle-plugin',
            generatedAt: new Date().toString(),
            project: [
                name: project.name,
                group: project.group?.toString() ?: '',
                version: project.version?.toString() ?: '',
                path: project.path
            ],
            configurations: []
        ]
        
        // Iterate through resolvable configurations
        project.configurations.each { Configuration config ->
            if (config.canBeResolved) {
                try {
                    def configData = [
                        name: config.name,
                        description: config.description ?: '',
                        dependencies: []
                    ]
                    
                    config.resolvedConfiguration.firstLevelModuleDependencies.each { ResolvedDependency dep ->
                        configData.dependencies.add(extractDependency(dep))
                    }
                    
                    if (!configData.dependencies.isEmpty()) {
                        graphData.configurations.add(configData)
                    }
                } catch (Exception e) {
                    logger.warn("Could not resolve configuration ${config.name}: ${e.message}")
                }
            }
        }
        
        // Count total dependencies
        def totalDeps = graphData.configurations.sum { it.dependencies.size() } ?: 0
        graphData.dependencyCount = totalDeps
        
        // Write JSON to file
        def outputFileObj = outputFile.get().asFile
        outputFileObj.parentFile.mkdirs()
        
        def gson = new GsonBuilder().setPrettyPrinting().create()
        outputFileObj.text = gson.toJson(graphData)
        
        logger.lifecycle("BazBOM: Generated dependency graph with ${totalDeps} dependencies")
        logger.lifecycle("BazBOM: Graph written to: ${outputFileObj.absolutePath}")
    }
    
    private Map extractDependency(ResolvedDependency dep) {
        return [
            group: dep.moduleGroup,
            name: dep.moduleName,
            version: dep.moduleVersion,
            configuration: dep.configuration,
            purl: "pkg:maven/${dep.moduleGroup}/${dep.moduleName}@${dep.moduleVersion}"
        ]
    }
}
