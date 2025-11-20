package com.bazbom.test;

import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import com.fasterxml.jackson.databind.ObjectMapper;
// NOTE: commons-collections is NOT imported - should be unreachable

public class App {
    private static final Logger logger = LogManager.getLogger(App.class);

    public static void main(String[] args) {
        // Using log4j - REACHABLE vulnerability
        logger.info("Starting application");

        // Using jackson - REACHABLE vulnerability
        ObjectMapper mapper = new ObjectMapper();
        logger.info("ObjectMapper created: {}", mapper.getClass().getName());

        // NOTE: commons-collections is never used
        // The vulnerability should be marked as UNREACHABLE

        logger.info("Application complete");
    }
}
