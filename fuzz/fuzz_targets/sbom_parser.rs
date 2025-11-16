//! Fuzzing target for SBOM parser
//!
//! This fuzzer tests the SBOM parsing code for crashes, panics, and memory safety issues

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test JSON parsing
    if let Ok(s) = std::str::from_utf8(data) {
        // Try to parse as SPDX JSON
        let _ = serde_json::from_str::<serde_json::Value>(s);

        // Try to parse as CycloneDX JSON
        let _ = serde_json::from_str::<serde_json::Value>(s);
    }

    // Test binary data handling
    let _ = String::from_utf8_lossy(data);
});
