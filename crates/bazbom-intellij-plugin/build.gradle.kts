plugins {
    id("org.jetbrains.kotlin.jvm") version "2.2.21"
    id("org.jetbrains.intellij") version "1.17.4"
}

group = "io.bazbom"
version = "1.0.0"

repositories {
    mavenCentral()
}

intellij {
    version.set("2025.2")
    type.set("IC")  // IntelliJ IDEA Community
    plugins.set(listOf("maven", "gradle", "Kotlin", "java"))
}

dependencies {
    implementation("com.fasterxml.jackson.core:jackson-databind:2.20.1")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.20.1")
}

tasks {
    patchPluginXml {
        sinceBuild.set("252")
        untilBuild.set("253.*")
        
        changeNotes.set("""
            <h3>1.0.0</h3>
            <ul>
              <li>Initial release</li>
              <li>Dependency tree visualization</li>
              <li>Real-time vulnerability highlighting</li>
              <li>One-click quick fixes</li>
              <li>Maven, Gradle, and Bazel support</li>
            </ul>
        """.trimIndent())
    }

    buildSearchableOptions {
        enabled = false
    }

    runIde {
        jvmArgs = listOf("-Xmx2048m")
    }
}

kotlin {
    jvmToolchain(21)
}
