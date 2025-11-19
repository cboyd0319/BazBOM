//! Java bytecode analysis for extracting method calls
//!
//! This module analyzes .class files to build a call graph.
//! For a full implementation, this would parse the bytecode constant pool
//! and method code to extract all method invocations.

use super::call_graph::CallGraph;
use super::entrypoints::is_entrypoint;
use super::error::Result;
use super::models::MethodNode;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Analyze Java .class files in a directory
pub fn analyze_classes(project_root: &Path, call_graph: &mut CallGraph) -> Result<()> {
    // Find all .class files
    for entry in WalkDir::new(project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip non-class files
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("class") {
            continue;
        }

        // Try to analyze this class file
        if let Err(e) = analyze_class_file(path, call_graph) {
            tracing::warn!("Failed to analyze {}: {}", path.display(), e);
        }
    }

    Ok(())
}

/// Analyze a single .class file using classfile-parser
fn analyze_class_file(class_path: &Path, call_graph: &mut CallGraph) -> Result<()> {
    use classfile_parser::class_parser;

    // Read the class file
    let class_bytes = fs::read(class_path)?;

    // Parse with classfile-parser
    let (_, class) = match class_parser(&class_bytes) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!("Failed to parse {}: {:?}", class_path.display(), e);
            return Ok(()); // Skip malformed classes
        }
    };

    // Get class name from constant pool
    let class_name = class.const_pool
        .get((class.this_class - 1) as usize)
        .and_then(|cp| {
            if let classfile_parser::constant_info::ConstantInfo::Class(c) = cp {
                class.const_pool.get((c.name_index - 1) as usize)
            } else {
                None
            }
        })
        .and_then(|cp| {
            if let classfile_parser::constant_info::ConstantInfo::Utf8(s) = cp {
                Some(s.utf8_string.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Unknown".to_string());

    // Extract all methods
    for method in &class.methods {
        // Get method name
        let method_name = class.const_pool
            .get((method.name_index - 1) as usize)
            .and_then(|cp| {
                if let classfile_parser::constant_info::ConstantInfo::Utf8(s) = cp {
                    Some(s.utf8_string.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        // Get method descriptor
        let descriptor = class.const_pool
            .get((method.descriptor_index - 1) as usize)
            .and_then(|cp| {
                if let classfile_parser::constant_info::ConstantInfo::Utf8(s) = cp {
                    Some(s.utf8_string.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "()V".to_string());

        // Create method node
        let method_id = format!("{}:{}{}", class_name, method_name, descriptor);
        let mut method_node = MethodNode::new(
            method_id.clone(),
            method_name.clone(),
            class_name.clone(),
            descriptor,
        );

        // Check access flags (classfile-parser provides a proper type)
        method_node.is_public = method.access_flags.contains(classfile_parser::method_info::MethodAccessFlags::PUBLIC);
        method_node.is_static = method.access_flags.contains(classfile_parser::method_info::MethodAccessFlags::STATIC);

        // Check if it's an entrypoint (main method, servlet methods, etc.)
        method_node.is_entrypoint = is_entrypoint(&method_node);

        // Extract method calls from bytecode
        let called_methods = extract_method_calls(&class, method);
        method_node.calls = called_methods;

        call_graph.add_method(method_node);
    }

    Ok(())
}

/// Extract method calls from bytecode instructions
fn extract_method_calls(
    class: &classfile_parser::ClassFile,
    method: &classfile_parser::method_info::MethodInfo,
) -> Vec<String> {
    use classfile_parser::attribute_info::code_attribute_parser;
    use classfile_parser::constant_info::ConstantInfo;

    let mut calls = Vec::new();

    // Find the Code attribute
    for attr in &method.attributes {
        // Check if this is a Code attribute by looking up the name
        let attr_name = class.const_pool
            .get((attr.attribute_name_index - 1) as usize)
            .and_then(|cp| {
                if let ConstantInfo::Utf8(s) = cp {
                    Some(s.utf8_string.as_str())
                } else {
                    None
                }
            });

        if attr_name == Some("Code") {
            // Parse the Code attribute from the raw bytes
            let code_attr = match code_attribute_parser(&attr.info) {
                Ok((_, code)) => code,
                Err(e) => {
                    tracing::debug!("Failed to parse Code attribute: {:?}", e);
                    continue;
                }
            };

            // Parse bytecode instructions
            let bytecode = &code_attr.code;
            let mut i = 0;

            while i < bytecode.len() {
                let opcode = bytecode[i];

                match opcode {
                    // invokevirtual - invoke instance method
                    0xb6 => {
                        if i + 2 < bytecode.len() {
                            let index = u16::from_be_bytes([bytecode[i + 1], bytecode[i + 2]]);
                            if let Some(method_ref) = resolve_method_ref(class, index) {
                                calls.push(method_ref);
                            }
                            i += 3;
                        } else {
                            break;
                        }
                    }
                    // invokespecial - invoke constructor or private method
                    0xb7 => {
                        if i + 2 < bytecode.len() {
                            let index = u16::from_be_bytes([bytecode[i + 1], bytecode[i + 2]]);
                            if let Some(method_ref) = resolve_method_ref(class, index) {
                                calls.push(method_ref);
                            }
                            i += 3;
                        } else {
                            break;
                        }
                    }
                    // invokestatic - invoke static method
                    0xb8 => {
                        if i + 2 < bytecode.len() {
                            let index = u16::from_be_bytes([bytecode[i + 1], bytecode[i + 2]]);
                            if let Some(method_ref) = resolve_method_ref(class, index) {
                                calls.push(method_ref);
                            }
                            i += 3;
                        } else {
                            break;
                        }
                    }
                    // invokeinterface - invoke interface method
                    0xb9 => {
                        if i + 4 < bytecode.len() {
                            let index = u16::from_be_bytes([bytecode[i + 1], bytecode[i + 2]]);
                            if let Some(method_ref) = resolve_method_ref(class, index) {
                                calls.push(method_ref);
                            }
                            i += 5; // invokeinterface has 2 extra bytes
                        } else {
                            break;
                        }
                    }
                    // invokedynamic - invoke dynamic method (lambdas, etc.)
                    0xba => {
                        if i + 4 < bytecode.len() {
                            // invokedynamic is complex, skip for now
                            i += 5;
                        } else {
                            break;
                        }
                    }
                    // For other opcodes, use instruction length table
                    _ => {
                        i += get_instruction_length(opcode, bytecode, i);
                    }
                }
            }
        }
    }

    calls
}

/// Resolve a method reference from the constant pool
fn resolve_method_ref(class: &classfile_parser::ClassFile, index: u16) -> Option<String> {
    use classfile_parser::constant_info::ConstantInfo;

    // Get the MethodRef/InterfaceMethodRef from constant pool
    let method_ref = class.const_pool.get((index - 1) as usize)?;

    let (class_index, name_and_type_index) = match method_ref {
        ConstantInfo::MethodRef(mr) => (mr.class_index, mr.name_and_type_index),
        ConstantInfo::InterfaceMethodRef(imr) => (imr.class_index, imr.name_and_type_index),
        _ => return None,
    };

    // Get class name
    let class_name = class
        .const_pool
        .get((class_index - 1) as usize)
        .and_then(|cp| {
            if let ConstantInfo::Class(c) = cp {
                class.const_pool.get((c.name_index - 1) as usize)
            } else {
                None
            }
        })
        .and_then(|cp| {
            if let ConstantInfo::Utf8(s) = cp {
                Some(s.utf8_string.clone())
            } else {
                None
            }
        })?;

    // Get method name and descriptor
    let name_and_type = class
        .const_pool
        .get((name_and_type_index - 1) as usize)?;

    if let ConstantInfo::NameAndType(nat) = name_and_type {
        let method_name = class
            .const_pool
            .get((nat.name_index - 1) as usize)
            .and_then(|cp| {
                if let ConstantInfo::Utf8(s) = cp {
                    Some(s.utf8_string.clone())
                } else {
                    None
                }
            })?;

        let descriptor = class
            .const_pool
            .get((nat.descriptor_index - 1) as usize)
            .and_then(|cp| {
                if let ConstantInfo::Utf8(s) = cp {
                    Some(s.utf8_string.clone())
                } else {
                    None
                }
            })?;

        return Some(format!("{}:{}{}", class_name, method_name, descriptor));
    }

    None
}

/// Get the length of a JVM instruction
fn get_instruction_length(opcode: u8, bytecode: &[u8], offset: usize) -> usize {
    match opcode {
        // Instructions with no operands (1 byte)
        0x00..=0x0f => 1, // nop, aconst_null, iconst_*, lconst_*, fconst_*, dconst_*
        0x1a..=0x35 => 1, // iload_*, lload_*, fload_*, dload_*, aload_*
        0x3b..=0x5f => 1, // istore_*, lstore_*, fstore_*, dstore_*, astore_*
        0x60..=0x83 => 1, // iadd, ladd, fadd, dadd, isub, lsub, fsub, dsub, etc.
        0x85..=0x98 => 1, // i2l, i2f, i2d, l2i, l2f, l2d, f2i, f2l, etc.
        0xac..=0xb1 => 1, // ireturn, lreturn, freturn, dreturn, areturn, return
        0xbe..=0xbf => 1, // arraylength, athrow
        0xc2..=0xc3 => 1, // monitorenter, monitorexit
        0xca..=0xcb => 1, // breakpoint, impdep1, impdep2

        // Instructions with 1 byte operand
        0x10 => 2,        // bipush
        0x12 => 2,        // ldc
        0x15..=0x19 => 2, // iload, lload, fload, dload, aload
        0x36..=0x3a => 2, // istore, lstore, fstore, dstore, astore
        0xa9 => 2,        // ret
        0xbc => 2,        // newarray

        // Instructions with 2 byte operands
        0x11 => 3,        // sipush
        0x13..=0x14 => 3, // ldc_w, ldc2_w
        0x84 => 3,        // iinc
        0x99..=0xa8 => 3, // ifeq, ifne, iflt, ifge, ifgt, ifle, if_icmpeq, etc., goto
        0xb2..=0xb5 => 3, // getstatic, putstatic, getfield, putfield
        0xb6..=0xb8 => 3, // invokevirtual, invokespecial, invokestatic
        0xbb => 3,        // new
        0xbd => 3,        // anewarray
        0xc0..=0xc1 => 3, // checkcast, instanceof

        // Special cases
        0xaa => {
            // tableswitch - complex padding
            let pad = (4 - ((offset + 1) % 4)) % 4;
            let low_offset = offset + 1 + pad + 4;
            let high_offset = low_offset + 4;
            if high_offset + 3 < bytecode.len() {
                let low = i32::from_be_bytes([
                    bytecode[low_offset],
                    bytecode[low_offset + 1],
                    bytecode[low_offset + 2],
                    bytecode[low_offset + 3],
                ]);
                let high = i32::from_be_bytes([
                    bytecode[high_offset],
                    bytecode[high_offset + 1],
                    bytecode[high_offset + 2],
                    bytecode[high_offset + 3],
                ]);
                let num_offsets = (high - low + 1) as usize;
                1 + pad + 12 + (num_offsets * 4)
            } else {
                1
            }
        }
        0xab => {
            // lookupswitch - complex padding
            let pad = (4 - ((offset + 1) % 4)) % 4;
            let npairs_offset = offset + 1 + pad + 4;
            if npairs_offset + 3 < bytecode.len() {
                let npairs = i32::from_be_bytes([
                    bytecode[npairs_offset],
                    bytecode[npairs_offset + 1],
                    bytecode[npairs_offset + 2],
                    bytecode[npairs_offset + 3],
                ]) as usize;
                1 + pad + 8 + (npairs * 8)
            } else {
                1
            }
        }
        0xb9 => 5, // invokeinterface
        0xba => 5, // invokedynamic
        0xc4 => {
            // wide - extends other instructions
            if offset + 1 < bytecode.len() {
                let next_opcode = bytecode[offset + 1];
                if next_opcode == 0x84 {
                    6 // wide iinc
                } else {
                    4 // wide load/store
                }
            } else {
                1
            }
        }
        0xc5 => 4,        // multianewarray
        0xc6..=0xc7 => 3, // ifnull, ifnonnull
        0xc8..=0xc9 => 5, // goto_w, jsr_w

        _ => 1, // Unknown or invalid opcode, skip 1 byte
    }
}

/// Analyze JAR files in a directory
pub fn analyze_jars(project_root: &Path, call_graph: &mut CallGraph) -> Result<()> {
    // Find all .jar files
    for entry in WalkDir::new(project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip non-JAR files
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("jar") {
            continue;
        }

        // Try to analyze this JAR file
        if let Err(e) = analyze_jar_file(path, call_graph) {
            tracing::warn!("Failed to analyze JAR {}: {}", path.display(), e);
        }
    }

    Ok(())
}

/// Analyze a single JAR file
fn analyze_jar_file(jar_path: &Path, call_graph: &mut CallGraph) -> Result<()> {
    // In a full implementation, we would:
    // 1. Open the JAR as a ZIP archive
    // 2. Extract all .class files
    // 3. For each class file, call analyze_class_file()

    // For now, this is a placeholder
    tracing::debug!("Would analyze JAR: {}", jar_path.display());

    // The real implementation would integrate with the existing shading.rs
    // functionality in the main bazbom crate which already has JAR parsing

    let _call_graph = call_graph; // Suppress unused warning
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_classes_empty_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let mut call_graph = CallGraph::new();

        let result = analyze_classes(temp_dir.path(), &mut call_graph);
        assert!(result.is_ok());
        assert_eq!(call_graph.methods.len(), 0);
    }

    #[test]
    fn test_bytecode_call_extraction() {
        // Test against our compiled Test.class
        let test_dir = std::path::Path::new("/tmp/java-reachability-test/target/classes");

        if !test_dir.exists() {
            println!("Skipping test - Test.class not found");
            return;
        }

        let mut call_graph = CallGraph::new();
        let result = analyze_classes(test_dir, &mut call_graph);

        assert!(result.is_ok(), "Failed to analyze classes: {:?}", result);

        println!("\n=== Call Graph ===");
        println!("Total methods: {}", call_graph.methods.len());

        for (method_id, method) in &call_graph.methods {
            println!("\n{}", method_id);
            println!("  Public: {}, Static: {}, Entrypoint: {}",
                     method.is_public, method.is_static, method.is_entrypoint);
            println!("  Calls {} methods:", method.calls.len());
            for call in &method.calls {
                println!("    -> {}", call);
            }
        }

        // Verify we found the main method
        let main_method = call_graph.methods.values()
            .find(|m| m.name == "main" && m.is_entrypoint);
        assert!(main_method.is_some(), "Should find main method as entrypoint");

        // Verify main calls used()
        let main = main_method.unwrap();
        assert!(main.calls.iter().any(|c| c.contains("used")),
                "main() should call used(), calls: {:?}", main.calls);
    }
}
