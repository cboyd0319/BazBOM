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
        
        // Focus on key configurations
        def keyConfigurations = ['compileClasspath', 'runtimeClasspath', 'testCompileClasspath', 'testRuntimeClasspath']
        
        // Iterate through resolvable configurations
        project.configurations.each { Configuration config ->
            // Skip if not a key configuration unless it contains 'runtime' or 'compile'
            boolean isKeyConfig = config.name in keyConfigurations || 
                                  config.name.toLowerCase().contains('runtime') || 
                                  config.name.toLowerCase().contains('compile')
            
            if (config.canBeResolved && isKeyConfig) {
                try {
                    def configData = [
                        name: config.name,
                        description: config.description ?: '',
                        dependencies: []
                    ]
                    
                    def resolvedConfig = config.resolvedConfiguration
                    def firstLevelDeps = resolvedConfig.firstLevelModuleDependencies
                    
                    logger.info("Processing configuration ${config.name} with ${firstLevelDeps.size()} dependencies")
                    
                    firstLevelDeps.each { ResolvedDependency dep ->
                        def depMap = [
                            group: dep.moduleGroup,
                            name: dep.moduleName,
                            version: dep.moduleVersion,
                            configuration: config.name,
                            purl: "pkg:maven/${dep.moduleGroup}/${dep.moduleName}@${dep.moduleVersion}".toString()
                        ]
                        configData.dependencies.add(depMap)
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
}
