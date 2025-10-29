package io.bazbom.reachability;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassVisitor;
import org.objectweb.asm.MethodVisitor;
import org.objectweb.asm.Opcodes;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.*;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;
import java.util.stream.Collectors;

/**
 * ASM-based reachability analyzer for BazBOM.
 * 
 * Analyzes bytecode to determine which methods and classes are reachable
 * from application entrypoints, enabling more accurate vulnerability assessment.
 */
public class Main {
    
    private static final String VERSION = "0.1.0";
    
    public static void main(String[] args) {
        try {
            List<String> argList = Arrays.asList(args);
            Path outputPath = Paths.get(findArgValue(argList, "--output", "reachability.json"));
            String classpath = findArgValue(argList, "--classpath", "");
            String entrypoints = findArgValue(argList, "--entrypoints", "");
            
            System.out.println("[reachability] Starting analysis");
            System.out.println("[reachability] Classpath: " + (classpath.isEmpty() ? "(empty)" : classpath));
            System.out.println("[reachability] Entrypoints: " + (entrypoints.isEmpty() ? "(auto-detect)" : entrypoints));
            
            ReachabilityResult result = analyzeReachability(classpath, entrypoints);
            
            // Write JSON output
            Files.createDirectories(outputPath.getParent() == null ? Paths.get(".") : outputPath.getParent());
            try (FileWriter writer = new FileWriter(outputPath.toFile())) {
                Gson gson = new GsonBuilder().setPrettyPrinting().create();
                gson.toJson(result, writer);
            }
            
            System.out.println("[reachability] Analysis complete");
            System.out.println("[reachability] Reachable methods: " + result.reachableMethods.size());
            System.out.println("[reachability] Reachable classes: " + result.reachableClasses.size());
            System.out.println("[reachability] Output: " + outputPath);
            
        } catch (Exception e) {
            System.err.println("[reachability] Error: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }
    
    private static ReachabilityResult analyzeReachability(String classpath, String entrypoints) throws IOException {
        ReachabilityResult result = new ReachabilityResult();
        result.tool = "bazbom-reachability";
        result.version = VERSION;
        result.classpath = classpath;
        result.entrypoints = entrypoints;
        
        if (classpath.isEmpty()) {
            System.out.println("[reachability] No classpath provided, returning empty result");
            return result;
        }
        
        try {
            // Parse classpath into file array
            List<File> classpathFiles = Arrays.stream(classpath.split(File.pathSeparator))
                    .map(String::trim)
                    .filter(s -> !s.isEmpty())
                    .map(File::new)
                    .filter(File::exists)
                    .collect(Collectors.toList());
            
            if (classpathFiles.isEmpty()) {
                System.out.println("[reachability] No valid classpath entries found");
                return result;
            }
            
            System.out.println("[reachability] Loading " + classpathFiles.size() + " classpath entries");
            
            // Build call graph
            Map<String, ClassInfo> classMap = new HashMap<>();
            for (File file : classpathFiles) {
                loadClassesFromFile(file, classMap);
            }
            
            System.out.println("[reachability] Loaded " + classMap.size() + " classes");
            
            // Find entrypoints
            Set<MethodRef> entrypointMethods = findEntrypoints(classMap, entrypoints);
            result.detectedEntrypoints = entrypointMethods.stream()
                    .map(MethodRef::toString)
                    .sorted()
                    .collect(Collectors.toList());
            
            System.out.println("[reachability] Found " + entrypointMethods.size() + " entrypoints");
            
            // Perform reachability analysis
            Set<MethodRef> reachableMethods = new HashSet<>();
            Set<String> reachableClasses = new HashSet<>();
            
            for (MethodRef entrypoint : entrypointMethods) {
                traverseCallGraph(classMap, entrypoint, reachableMethods, reachableClasses);
            }
            
            // Convert to output format
            result.reachableMethods = reachableMethods.stream()
                    .map(MethodRef::toString)
                    .sorted()
                    .collect(Collectors.toList());
            
            result.reachableClasses = new ArrayList<>(reachableClasses);
            Collections.sort(result.reachableClasses);
            
            result.reachablePackages = extractPackages(reachableClasses);
            
        } catch (Exception e) {
            System.err.println("[reachability] Analysis error: " + e.getMessage());
            e.printStackTrace();
            result.error = e.getMessage();
        }
        
        return result;
    }
    
    private static void loadClassesFromFile(File file, Map<String, ClassInfo> classMap) throws IOException {
        if (file.isDirectory()) {
            loadClassesFromDirectory(file, file, classMap);
        } else if (file.getName().endsWith(".jar")) {
            loadClassesFromJar(file, classMap);
        } else if (file.getName().endsWith(".class")) {
            loadSingleClass(file, classMap);
        }
    }
    
    private static void loadClassesFromDirectory(File root, File dir, Map<String, ClassInfo> classMap) throws IOException {
        File[] files = dir.listFiles();
        if (files == null) return;
        
        for (File file : files) {
            if (file.isDirectory()) {
                loadClassesFromDirectory(root, file, classMap);
            } else if (file.getName().endsWith(".class")) {
                try (InputStream is = Files.newInputStream(file.toPath())) {
                    loadClass(is, classMap);
                }
            }
        }
    }
    
    private static void loadClassesFromJar(File jarFile, Map<String, ClassInfo> classMap) throws IOException {
        try (JarFile jar = new JarFile(jarFile)) {
            java.util.Enumeration<JarEntry> entries = jar.entries();
            while (entries.hasMoreElements()) {
                JarEntry entry = entries.nextElement();
                if (entry.getName().endsWith(".class") && !entry.isDirectory()) {
                    try (InputStream is = jar.getInputStream(entry)) {
                        loadClass(is, classMap);
                    } catch (Exception e) {
                        // Skip problematic classes
                    }
                }
            }
        }
    }
    
    private static void loadSingleClass(File classFile, Map<String, ClassInfo> classMap) throws IOException {
        try (InputStream is = Files.newInputStream(classFile.toPath())) {
            loadClass(is, classMap);
        }
    }
    
    private static void loadClass(InputStream is, Map<String, ClassInfo> classMap) throws IOException {
        ClassReader reader = new ClassReader(is);
        ClassInfo classInfo = new ClassInfo();
        classInfo.name = reader.getClassName().replace('/', '.');
        
        reader.accept(new ClassVisitor(Opcodes.ASM9) {
            @Override
            public MethodVisitor visitMethod(int access, String name, String descriptor, 
                                            String signature, String[] exceptions) {
                MethodInfo methodInfo = new MethodInfo();
                methodInfo.name = name;
                methodInfo.descriptor = descriptor;
                methodInfo.isPublic = (access & Opcodes.ACC_PUBLIC) != 0;
                methodInfo.isStatic = (access & Opcodes.ACC_STATIC) != 0;
                classInfo.methods.add(methodInfo);
                
                return new MethodVisitor(Opcodes.ASM9) {
                    @Override
                    public void visitMethodInsn(int opcode, String owner, String name, 
                                               String descriptor, boolean isInterface) {
                        String targetClass = owner.replace('/', '.');
                        MethodRef ref = new MethodRef(targetClass, name, descriptor);
                        methodInfo.calls.add(ref);
                    }
                };
            }
        }, 0);
        
        classMap.put(classInfo.name, classInfo);
    }
    
    private static Set<MethodRef> findEntrypoints(Map<String, ClassInfo> classMap, String entrypointsArg) {
        Set<MethodRef> entrypoints = new HashSet<>();
        
        if (!entrypointsArg.isEmpty()) {
            // Parse specified entrypoints
            String[] specified = entrypointsArg.split(",");
            for (String ep : specified) {
                ep = ep.trim();
                System.out.println("[reachability] Looking for entrypoint: " + ep);
                // Simple format: "com.example.Main.main"
                int lastDot = ep.lastIndexOf('.');
                if (lastDot > 0) {
                    String className = ep.substring(0, lastDot);
                    String methodName = ep.substring(lastDot + 1);
                    ClassInfo classInfo = classMap.get(className);
                    if (classInfo != null) {
                        for (MethodInfo method : classInfo.methods) {
                            if (method.name.equals(methodName)) {
                                entrypoints.add(new MethodRef(className, method.name, method.descriptor));
                            }
                        }
                    }
                }
            }
        }
        
        // Auto-detect main methods and public constructors
        for (ClassInfo classInfo : classMap.values()) {
            for (MethodInfo method : classInfo.methods) {
                if (isEntrypoint(method)) {
                    entrypoints.add(new MethodRef(classInfo.name, method.name, method.descriptor));
                }
            }
        }
        
        return entrypoints;
    }
    
    private static boolean isEntrypoint(MethodInfo method) {
        // Main methods: public static void main(String[])
        if (method.name.equals("main") && 
            method.isPublic && 
            method.isStatic &&
            method.descriptor.equals("([Ljava/lang/String;)V")) {
            return true;
        }
        
        // Public constructors (for library analysis)
        if (method.name.equals("<init>") && method.isPublic) {
            return true;
        }
        
        return false;
    }
    
    private static void traverseCallGraph(Map<String, ClassInfo> classMap, MethodRef method, 
                                         Set<MethodRef> reachableMethods, 
                                         Set<String> reachableClasses) {
        if (reachableMethods.contains(method)) {
            return; // Already visited
        }
        
        reachableMethods.add(method);
        reachableClasses.add(method.className);
        
        // Find the method in the class map
        ClassInfo classInfo = classMap.get(method.className);
        if (classInfo != null) {
            for (MethodInfo methodInfo : classInfo.methods) {
                if (methodInfo.name.equals(method.methodName) && 
                    methodInfo.descriptor.equals(method.descriptor)) {
                    // Traverse all calls from this method
                    for (MethodRef callee : methodInfo.calls) {
                        traverseCallGraph(classMap, callee, reachableMethods, reachableClasses);
                    }
                }
            }
        }
    }
    
    private static List<String> extractPackages(Set<String> classes) {
        Set<String> packages = new HashSet<>();
        for (String className : classes) {
            int lastDot = className.lastIndexOf('.');
            if (lastDot > 0) {
                packages.add(className.substring(0, lastDot));
            }
        }
        List<String> result = new ArrayList<>(packages);
        Collections.sort(result);
        return result;
    }
    
    private static String findArgValue(List<String> args, String key, String def) {
        int idx = args.indexOf(key);
        if (idx >= 0 && idx + 1 < args.size()) {
            return args.get(idx + 1);
        }
        return def;
    }
    
    // JSON output structure
    static class ReachabilityResult {
        String tool;
        String version;
        String classpath;
        String entrypoints;
        List<String> detectedEntrypoints = new ArrayList<>();
        List<String> reachableMethods = new ArrayList<>();
        List<String> reachableClasses = new ArrayList<>();
        List<String> reachablePackages = new ArrayList<>();
        String error;
    }
    
    // Internal data structures
    static class ClassInfo {
        String name;
        List<MethodInfo> methods = new ArrayList<>();
    }
    
    static class MethodInfo {
        String name;
        String descriptor;
        boolean isPublic;
        boolean isStatic;
        List<MethodRef> calls = new ArrayList<>();
    }
    
    static class MethodRef {
        String className;
        String methodName;
        String descriptor;
        
        MethodRef(String className, String methodName, String descriptor) {
            this.className = className;
            this.methodName = methodName;
            this.descriptor = descriptor;
        }
        
        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            MethodRef that = (MethodRef) o;
            return Objects.equals(className, that.className) &&
                   Objects.equals(methodName, that.methodName) &&
                   Objects.equals(descriptor, that.descriptor);
        }
        
        @Override
        public int hashCode() {
            return Objects.hash(className, methodName, descriptor);
        }
        
        @Override
        public String toString() {
            return className + "." + methodName + descriptor;
        }
    }
}

