package com.example.bazbom

import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.runApplication

/**
 * Main application class for the BazBOM Gradle Kotlin example.
 * This is a simple Spring Boot application written in Kotlin used to test BazBOM's Gradle support.
 */
@SpringBootApplication
class Application

fun main(args: Array<String>) {
    runApplication<Application>(*args)
}
