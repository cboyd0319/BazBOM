package com.example.lib;

import com.example.common.StringUtils;
import com.google.gson.Gson;
import org.apache.commons.lang3.StringUtils as CommonsStringUtils;

import java.util.HashMap;
import java.util.Map;

/**
 * Processes data with JSON serialization capabilities.
 */
public class DataProcessor {
    
    private final Gson gson = new Gson();
    
    /**
     * Processes input data and returns formatted output.
     */
    public String processData(String input) {
        if (StringUtils.isNullOrEmpty(input)) {
            return "No data";
        }
        
        String capitalized = CommonsStringUtils.capitalize(input);
        String padded = StringUtils.padEnd(capitalized, 20);
        
        Map<String, String> result = new HashMap<>();
        result.put("original", input);
        result.put("processed", padded);
        
        return gson.toJson(result);
    }
}
