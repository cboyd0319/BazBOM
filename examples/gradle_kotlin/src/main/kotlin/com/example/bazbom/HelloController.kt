package com.example.bazbom

import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.RestController

/**
 * Simple REST controller for testing.
 */
@RestController
class HelloController {
    
    @GetMapping("/")
    fun hello(): String = "BazBOM Gradle Kotlin Example is running!"
    
    @GetMapping("/health")
    fun health(): String = "OK"
    
    @GetMapping("/info")
    fun info(): Map<String, String> = mapOf(
        "name" to "BazBOM Gradle Kotlin Example",
        "version" to "1.0.0",
        "language" to "Kotlin"
    )
}
