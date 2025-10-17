package com.example.app;

import com.example.lib.DataProcessor;

/**
 * Main application entry point.
 */
public class Application {
    
    public static void main(String[] args) {
        DataProcessor processor = new DataProcessor();
        
        String input = args.length > 0 ? args[0] : "hello world";
        String result = processor.processData(input);
        
        System.out.println("Processing result:");
        System.out.println(result);
    }
}
