package io.bazbom.reachability;

import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

/**
 * OPAL-based reachability tool skeleton.
 * 
 * This is a placeholder that accepts inputs and emits a minimal JSON result.
 * Integration with OPAL will be added in a subsequent iteration.
 */
public class Main {
    public static void main(String[] args) throws IOException {
        List<String> argList = Arrays.asList(args);
        Path out = Paths.get(findArgValue(argList, "--output", "reachability.json"));
        String classpath = findArgValue(argList, "--classpath", "");
        String entrypoints = findArgValue(argList, "--entrypoints", "");

        Files.createDirectories(out.getParent() == null ? Paths.get(".") : out.getParent());
        try (FileWriter fw = new FileWriter(out.toFile())) {
            fw.write("{\n");
            fw.write("  \"tool\": \"bazbom-reachability\",\n");
            fw.write("  \"version\": \"0.0.1-dev\",\n");
            fw.write("  \"classpath\": \"" + escape(classpath) + "\",\n");
            fw.write("  \"entrypoints\": \"" + escape(entrypoints) + "\",\n");
            fw.write("  \"reachable\": []\n");
            fw.write("}\n");
        }
        System.out.println("[reachability] wrote " + out);
    }

    private static String findArgValue(List<String> args, String key, String def) {
        int idx = args.indexOf(key);
        if (idx >= 0 && idx + 1 < args.size()) {
            return args.get(idx + 1);
        }
        return def;
    }

    private static String escape(String s) {
        return s.replace("\\", "\\\\").replace("\"", "\\\"");
    }
}

