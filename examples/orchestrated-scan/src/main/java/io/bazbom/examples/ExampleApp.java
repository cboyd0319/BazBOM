package io.bazbom.examples;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;

/**
 * Example application demonstrating BazBOM's orchestrated scan capabilities.
 * 
 * This code intentionally uses weak cryptography to demonstrate Semgrep detection.
 */
public class ExampleApp {
    public static void main(String[] args) throws NoSuchAlgorithmException {
        System.out.println("BazBOM Orchestrated Scan Example");
        
        // This will be detected by Semgrep as weak crypto (MD5)
        MessageDigest md5 = MessageDigest.getInstance("MD5");
        byte[] hash = md5.digest("example".getBytes());
        
        System.out.println("Hash length: " + hash.length);
    }
}
