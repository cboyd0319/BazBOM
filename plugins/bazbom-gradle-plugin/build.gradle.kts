plugins {
    `java-gradle-plugin`
    `groovy`
    `maven-publish`
}

group = "io.bazbom"
version = "0.1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation(gradleApi())
    implementation(localGroovy())
    implementation("com.google.code.gson:gson:2.10.1")
    
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
    testImplementation("org.spockframework:spock-core:2.3-groovy-3.0")
}

gradlePlugin {
    plugins {
        create("bazbomPlugin") {
            id = "io.bazbom.gradle-plugin"
            implementationClass = "io.bazbom.gradle.BazBomPlugin"
            displayName = "BazBOM Gradle Plugin"
            description = "Generate authoritative dependency graphs for BazBOM analysis"
        }
    }
}

tasks.test {
    useJUnitPlatform()
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}
