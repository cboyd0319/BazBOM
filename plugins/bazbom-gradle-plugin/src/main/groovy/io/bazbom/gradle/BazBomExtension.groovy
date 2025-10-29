package io.bazbom.gradle

import org.gradle.api.provider.Property

/**
 * Extension for configuring the BazBOM Gradle Plugin.
 */
abstract class BazBomExtension {
    
    /**
     * Whether to include test configurations in the dependency graph.
     */
    abstract Property<Boolean> getIncludeTestConfigurations()
    
    /**
     * Whether to include Android variants in the dependency graph.
     */
    abstract Property<Boolean> getIncludeAndroidVariants()
    
    /**
     * Whether to analyze Shadow plugin configurations for shading.
     */
    abstract Property<Boolean> getAnalyzeShadow()
    
    /**
     * Output directory for generated files.
     */
    abstract Property<File> getOutputDirectory()
    
    BazBomExtension() {
        includeTestConfigurations.convention(true)
        includeAndroidVariants.convention(true)
        analyzeShadow.convention(true)
    }
}
