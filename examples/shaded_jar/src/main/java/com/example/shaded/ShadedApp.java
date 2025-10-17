package com.example.shaded;

import com.google.common.collect.ImmutableList;
import org.apache.commons.text.WordUtils;

/**
 * Example application that will be packaged as a shaded/fat JAR.
 * All dependencies will be included in the JAR.
 */
public class ShadedApp {
    
    public static void main(String[] args) {
        // Use Guava
        ImmutableList<String> words = ImmutableList.of(
            "supply", "chain", "security"
        );
        
        System.out.println("Processing words:");
        for (String word : words) {
            // Use Apache Commons Text
            String capitalized = WordUtils.capitalize(word);
            System.out.println("  - " + capitalized);
        }
        
        System.out.println("\nThis application can be packaged as a fat/shaded JAR");
        System.out.println("with all dependencies included for easy deployment.");
    }
}
