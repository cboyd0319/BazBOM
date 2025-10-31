plugins {
    id("org.jetbrains.kotlin.jvm") version "1.9.20"
    id("org.jetbrains.intellij") version "1.16.0"
}

group = "io.bazbom"
version = "1.0.0"

repositories {
    mavenCentral()
}

intellij {
    version.set("2023.3")
    type.set("IC")  // IntelliJ IDEA Community
    plugins.set(listOf("maven", "gradle", "Kotlin"))
}

dependencies {
    implementation("com.fasterxml.jackson.core:jackson-databind:2.15.2")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.15.2")
}

tasks {
    patchPluginXml {
        sinceBuild.set("233")
        untilBuild.set("241.*")
        
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
    jvmToolchain(17)
}
