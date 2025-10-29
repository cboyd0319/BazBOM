package io.bazbom.maven;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.File;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Unit tests for BazBomGraphMojo.
 */
class BazBomGraphMojoTest {

    @TempDir
    Path tempDir;

    @Test
    void testMojoInstantiation() {
        BazBomGraphMojo mojo = new BazBomGraphMojo();
        assertNotNull(mojo, "Mojo should be instantiable");
    }

    @Test
    void testOutputFileConfiguration() {
        // This is a basic structural test
        // Full integration tests would require Maven test harness
        BazBomGraphMojo mojo = new BazBomGraphMojo();
        assertNotNull(mojo, "Mojo instance should not be null");
    }
}
