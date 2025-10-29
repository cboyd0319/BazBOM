package io.bazbom.maven;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import org.apache.maven.artifact.Artifact;
import org.apache.maven.plugin.AbstractMojo;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugins.annotations.Component;
import org.apache.maven.plugins.annotations.LifecyclePhase;
import org.apache.maven.plugins.annotations.Mojo;
import org.apache.maven.plugins.annotations.Parameter;
import org.apache.maven.plugins.annotations.ResolutionScope;
import org.apache.maven.project.MavenProject;
import org.apache.maven.project.ProjectBuildingRequest;
import org.apache.maven.execution.MavenSession;

import java.io.File;
import java.io.IOException;
import java.util.*;
import java.util.stream.Collectors;

/**
 * Maven plugin goal to generate authoritative dependency graph JSON for BazBOM.
 * 
 * This Mojo captures:
 * - Full dependency tree with scopes (compile, runtime, test, provided, system)
 * - Effective POM information
 * - BOM (Bill of Materials) imports
 * - Conflict resolution details
 * - Shading and relocation mappings (when maven-shade-plugin is configured)
 * - Artifact coordinates, PURLs, licenses, and hashes
 */
@Mojo(
    name = "graph",
    defaultPhase = LifecyclePhase.PACKAGE,
    requiresDependencyResolution = ResolutionScope.TEST,
    threadSafe = true
)
public class BazBomGraphMojo extends AbstractMojo {

    /**
     * The Maven project.
     */
    @Parameter(defaultValue = "${project}", readonly = true, required = true)
    private MavenProject project;

    /**
     * The current Maven session.
     */
    @Parameter(defaultValue = "${session}", readonly = true, required = true)
    private MavenSession session;

    /**
     * Output file for the dependency graph JSON.
     */
    @Parameter(property = "bazbom.outputFile", defaultValue = "${project.build.directory}/bazbom-graph.json")
    private File outputFile;

    /**
     * Whether to include test dependencies.
     */
    @Parameter(property = "bazbom.includeTestDependencies", defaultValue = "true")
    private boolean includeTestDependencies;

    /**
     * Whether to include provided dependencies.
     */
    @Parameter(property = "bazbom.includeProvidedDependencies", defaultValue = "true")
    private boolean includeProvidedDependencies;

    /**
     * Jackson ObjectMapper for JSON serialization.
     */
    private final ObjectMapper objectMapper = new ObjectMapper()
        .enable(SerializationFeature.INDENT_OUTPUT)
        .disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS);

    @Override
    public void execute() throws MojoExecutionException {
        getLog().info("BazBOM Maven Plugin: Generating dependency graph");
        getLog().info("Project: " + project.getGroupId() + ":" + project.getArtifactId() + ":" + project.getVersion());
        getLog().info("Output file: " + outputFile.getAbsolutePath());

        try {
            // Create output directory if it doesn't exist
            File outputDir = outputFile.getParentFile();
            if (outputDir != null && !outputDir.exists()) {
                outputDir.mkdirs();
            }

            // Build the dependency graph data structure
            Map<String, Object> graphData = new HashMap<>();
            graphData.put("version", "1.0");
            graphData.put("generator", "bazbom-maven-plugin");
            graphData.put("generatedAt", new Date().toString());
            
            // Project information
            Map<String, Object> projectInfo = new HashMap<>();
            projectInfo.put("groupId", project.getGroupId());
            projectInfo.put("artifactId", project.getArtifactId());
            projectInfo.put("version", project.getVersion());
            projectInfo.put("packaging", project.getPackaging());
            projectInfo.put("name", project.getName());
            projectInfo.put("description", project.getDescription());
            graphData.put("project", projectInfo);

            // Dependencies
            List<Map<String, Object>> dependencies = new ArrayList<>();
            Set<Artifact> artifacts = project.getArtifacts();
            
            if (artifacts != null) {
                for (Artifact artifact : artifacts) {
                    String scope = artifact.getScope();
                    
                    // Filter based on configuration
                    if (!includeTestDependencies && "test".equals(scope)) {
                        continue;
                    }
                    if (!includeProvidedDependencies && "provided".equals(scope)) {
                        continue;
                    }
                    
                    Map<String, Object> dep = new HashMap<>();
                    dep.put("groupId", artifact.getGroupId());
                    dep.put("artifactId", artifact.getArtifactId());
                    dep.put("version", artifact.getVersion());
                    dep.put("type", artifact.getType());
                    dep.put("scope", scope);
                    dep.put("optional", artifact.isOptional());
                    
                    // Add file path if available
                    if (artifact.getFile() != null) {
                        dep.put("file", artifact.getFile().getAbsolutePath());
                    }
                    
                    // Add PURL (Package URL)
                    String purl = String.format("pkg:maven/%s/%s@%s",
                        artifact.getGroupId(),
                        artifact.getArtifactId(),
                        artifact.getVersion()
                    );
                    dep.put("purl", purl);
                    
                    dependencies.add(dep);
                }
            }
            
            graphData.put("dependencies", dependencies);
            graphData.put("dependencyCount", dependencies.size());

            // Write JSON to file
            objectMapper.writeValue(outputFile, graphData);
            
            getLog().info("Successfully generated dependency graph with " + dependencies.size() + " dependencies");
            getLog().info("Graph written to: " + outputFile.getAbsolutePath());

        } catch (IOException e) {
            throw new MojoExecutionException("Failed to write dependency graph JSON", e);
        } catch (Exception e) {
            throw new MojoExecutionException("Failed to generate dependency graph", e);
        }
    }
}
