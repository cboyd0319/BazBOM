package com.example.common;

import com.google.common.base.Strings;

/**
 * Common string utility functions.
 */
public class StringUtils {
    
    /**
     * Safely pads a string to the specified length.
     */
    public static String padEnd(String input, int length) {
        return Strings.padEnd(input, length, ' ');
    }
    
    /**
     * Checks if a string is null or empty.
     */
    public static boolean isNullOrEmpty(String input) {
        return Strings.isNullOrEmpty(input);
    }
}
