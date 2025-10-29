package io.bazbom.gradle

import org.gradle.testkit.runner.GradleRunner
import org.gradle.testkit.runner.TaskOutcome
import spock.lang.Specification
import spock.lang.TempDir

import java.nio.file.Files
import java.nio.file.Path

class BazBomPluginFunctionalTest extends Specification {

    @TempDir
    Path testProjectDir

    File buildFile
    File settingsFile

    def setup() {
        settingsFile = testProjectDir.resolve('settings.gradle').toFile()
        settingsFile << """
            rootProject.name = 'test-project'
        """

        buildFile = testProjectDir.resolve('build.gradle').toFile()
    }

    def "can apply plugin"() {
        given:
        buildFile << """
            plugins {
                id 'io.bazbom.gradle-plugin'
            }
        """

        when:
        def result = GradleRunner.create()
            .withProjectDir(testProjectDir.toFile())
            .withArguments('tasks', '--group=bazbom')
            .withPluginClasspath()
            .build()

        then:
        result.output.contains('bazbomGraph')
        result.output.contains('bazbomSbom')
        result.output.contains('bazbomFindings')
    }

    def "generates dependency graph for simple project"() {
        given:
        buildFile << """
            plugins {
                id 'java'
                id 'io.bazbom.gradle-plugin'
            }
            
            repositories {
                mavenCentral()
            }
            
            dependencies {
                implementation 'com.google.guava:guava:32.1.3-jre'
                testImplementation 'junit:junit:4.13.2'
            }
        """

        when:
        def result = GradleRunner.create()
            .withProjectDir(testProjectDir.toFile())
            .withArguments('bazbomGraph')
            .withPluginClasspath()
            .build()

        then:
        result.task(':bazbomGraph').outcome == TaskOutcome.SUCCESS
        
        def graphFile = testProjectDir.resolve('build/bazbom-graph.json').toFile()
        graphFile.exists()
        
        def graphContent = graphFile.text
        graphContent.contains('"generator": "bazbom-gradle-plugin"')
        graphContent.contains('"name": "test-project"')
        graphContent.contains('guava')
    }

    def "graph includes multiple configurations"() {
        given:
        buildFile << """
            plugins {
                id 'java'
                id 'io.bazbom.gradle-plugin'
            }
            
            repositories {
                mavenCentral()
            }
            
            dependencies {
                implementation 'org.apache.commons:commons-lang3:3.13.0'
                testImplementation 'org.junit.jupiter:junit-jupiter:5.10.0'
            }
        """

        when:
        def result = GradleRunner.create()
            .withProjectDir(testProjectDir.toFile())
            .withArguments('bazbomGraph')
            .withPluginClasspath()
            .build()

        then:
        result.task(':bazbomGraph').outcome == TaskOutcome.SUCCESS
        
        def graphFile = testProjectDir.resolve('build/bazbom-graph.json').toFile()
        def graphContent = graphFile.text
        
        graphContent.contains('compileClasspath')
        graphContent.contains('runtimeClasspath')
        graphContent.contains('testCompileClasspath')
        graphContent.contains('testRuntimeClasspath')
    }

    def "graph includes PURL for dependencies"() {
        given:
        buildFile << """
            plugins {
                id 'java'
                id 'io.bazbom.gradle-plugin'
            }
            
            repositories {
                mavenCentral()
            }
            
            dependencies {
                implementation 'com.google.guava:guava:32.1.3-jre'
            }
        """

        when:
        def result = GradleRunner.create()
            .withProjectDir(testProjectDir.toFile())
            .withArguments('bazbomGraph')
            .withPluginClasspath()
            .build()

        then:
        result.task(':bazbomGraph').outcome == TaskOutcome.SUCCESS
        
        def graphFile = testProjectDir.resolve('build/bazbom-graph.json').toFile()
        def graphContent = graphFile.text
        
        graphContent.contains('pkg:maven/com.google.guava/guava@32.1.3-jre')
    }

    def "bazbomSbom task depends on bazbomGraph"() {
        given:
        buildFile << """
            plugins {
                id 'java'
                id 'io.bazbom.gradle-plugin'
            }
        """

        when:
        def result = GradleRunner.create()
            .withProjectDir(testProjectDir.toFile())
            .withArguments('bazbomSbom', '--dry-run')
            .withPluginClasspath()
            .build()

        then:
        result.output.contains(':bazbomGraph')
    }

    def "bazbomFindings task depends on bazbomGraph"() {
        given:
        buildFile << """
            plugins {
                id 'java'
                id 'io.bazbom.gradle-plugin'
            }
        """

        when:
        def result = GradleRunner.create()
            .withProjectDir(testProjectDir.toFile())
            .withArguments('bazbomFindings', '--dry-run')
            .withPluginClasspath()
            .build()

        then:
        result.output.contains(':bazbomGraph')
    }

    def "extension configuration is applied"() {
        given:
        buildFile << """
            plugins {
                id 'java'
                id 'io.bazbom.gradle-plugin'
            }
            
            bazbom {
                includeTestConfigurations = false
            }
            
            repositories {
                mavenCentral()
            }
            
            dependencies {
                implementation 'com.google.guava:guava:32.1.3-jre'
            }
        """

        when:
        def result = GradleRunner.create()
            .withProjectDir(testProjectDir.toFile())
            .withArguments('bazbomGraph')
            .withPluginClasspath()
            .build()

        then:
        result.task(':bazbomGraph').outcome == TaskOutcome.SUCCESS
        
        def graphFile = testProjectDir.resolve('build/bazbom-graph.json').toFile()
        graphFile.exists()
    }
}
