package io.bazbom.reachability;

import org.junit.Test;
import static org.junit.Assert.*;

import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

import com.google.gson.Gson;

public class MainTest {

    @Test
    public void testEmptyClasspath() throws Exception {
        Path tempFile = Files.createTempFile("reachability-test", ".json");
        
        Main.main(new String[] {
            "--output", tempFile.toString(),
            "--classpath", "",
            "--entrypoints", ""
        });
        
        assertTrue("Output file should exist", Files.exists(tempFile));
        
        String json = new String(Files.readAllBytes(tempFile));
        assertNotNull("JSON output should not be null", json);
        assertTrue("JSON should contain tool name", json.contains("bazbom-reachability"));
        
        Gson gson = new Gson();
        Main.ReachabilityResult result = gson.fromJson(json, Main.ReachabilityResult.class);
        
        assertEquals("bazbom-reachability", result.tool);
        assertNotNull("Version should be set", result.version);
        assertEquals(0, result.reachableMethods.size());
        assertEquals(0, result.reachableClasses.size());
        
        Files.deleteIfExists(tempFile);
    }
    
    @Test
    public void testNonExistentClasspath() throws Exception {
        Path tempFile = Files.createTempFile("reachability-test", ".json");
        
        Main.main(new String[] {
            "--output", tempFile.toString(),
            "--classpath", "/nonexistent/path.jar",
            "--entrypoints", ""
        });
        
        assertTrue("Output file should exist", Files.exists(tempFile));
        
        String json = new String(Files.readAllBytes(tempFile));
        Gson gson = new Gson();
        Main.ReachabilityResult result = gson.fromJson(json, Main.ReachabilityResult.class);
        
        assertEquals("bazbom-reachability", result.tool);
        assertEquals(0, result.reachableMethods.size());
        
        Files.deleteIfExists(tempFile);
    }
    
    @Test
    public void testOutputFileCreation() throws Exception {
        Path tempDir = Files.createTempDirectory("reachability-test");
        Path outputFile = tempDir.resolve("subdir").resolve("output.json");
        
        Main.main(new String[] {
            "--output", outputFile.toString(),
            "--classpath", "",
            "--entrypoints", ""
        });
        
        assertTrue("Output file should be created with parent directories", Files.exists(outputFile));
        
        Files.deleteIfExists(outputFile);
        Files.deleteIfExists(outputFile.getParent());
        Files.deleteIfExists(tempDir);
    }
    
    @Test
    public void testDefaultArguments() throws Exception {
        // Test that the tool runs with minimal arguments
        Path tempFile = Files.createTempFile("reachability-test", ".json");
        
        Main.main(new String[] {
            "--output", tempFile.toString()
        });
        
        assertTrue("Output file should exist", Files.exists(tempFile));
        
        String json = new String(Files.readAllBytes(tempFile));
        assertTrue("JSON should be valid", json.contains("reachabilityResult") || json.contains("tool"));
        
        Files.deleteIfExists(tempFile);
    }
    
    @Test
    public void testMethodRefEquals() {
        Main.MethodRef ref1 = new Main.MethodRef("com.example.Test", "method", "()V");
        Main.MethodRef ref2 = new Main.MethodRef("com.example.Test", "method", "()V");
        Main.MethodRef ref3 = new Main.MethodRef("com.example.Test", "method", "(I)V");
        Main.MethodRef ref4 = new Main.MethodRef("com.example.Other", "method", "()V");
        
        assertEquals("Same methods should be equal", ref1, ref2);
        assertNotEquals("Different descriptors should not be equal", ref1, ref3);
        assertNotEquals("Different classes should not be equal", ref1, ref4);
        
        assertEquals("HashCodes should match for equal objects", ref1.hashCode(), ref2.hashCode());
    }
    
    @Test
    public void testMethodRefToString() {
        Main.MethodRef ref = new Main.MethodRef("com.example.Test", "myMethod", "(Ljava/lang/String;)I");
        String str = ref.toString();
        
        assertTrue("ToString should contain class name", str.contains("com.example.Test"));
        assertTrue("ToString should contain method name", str.contains("myMethod"));
        assertTrue("ToString should contain descriptor", str.contains("(Ljava/lang/String;)I"));
    }
}
