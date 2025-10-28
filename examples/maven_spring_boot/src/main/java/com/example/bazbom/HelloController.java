package com.example.bazbom;

import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * Simple REST controller for testing.
 */
@RestController
public class HelloController {
    
    @GetMapping("/")
    public String hello() {
        return "BazBOM Maven Example is running!";
    }
    
    @GetMapping("/health")
    public String health() {
        return "OK";
    }
}
