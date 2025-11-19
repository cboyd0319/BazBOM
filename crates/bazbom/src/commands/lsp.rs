//! LSP server command handler

use anyhow::Result;

/// Handle LSP server command
pub fn handle_lsp() -> Result<()> {
    println!("BazBOM LSP Server");
    println!("=================\n");

    println!("The BazBOM LSP server provides real-time vulnerability scanning in your IDE.\n");

    println!("INSTALLATION:");
    println!("  The LSP server is a separate binary. Install with:");
    println!("  cargo install --path crates/bazbom-lsp");
    println!("  OR");
    println!("  cargo install bazbom-lsp  (when published)\n");

    println!("USAGE:");
    println!("  Run the LSP server directly:");
    println!("  $ bazbom-lsp\n");

    println!("VS CODE SETUP:");
    println!("  1. Install the 'Custom Language Server' extension");
    println!("  2. Add to settings.json:");
    println!("     {{");
    println!("       \"customLanguageServerExtension.commands\": [{{");
    println!("         \"id\": \"bazbom\",");
    println!("         \"name\": \"BazBOM Security\",");
    println!("         \"command\": \"bazbom-lsp\",");
    println!("         \"languages\": [\"xml\", \"groovy\", \"kotlin\", \"starlark\"]");
    println!("       }}]");
    println!("     }}\n");

    println!("INTELLIJ SETUP:");
    println!("  1. Install the 'LSP4IJ' plugin");
    println!("  2. Go to Settings > Languages > LSP");
    println!("  3. Add server: bazbom-lsp");
    println!("  4. Associate with file types: *.xml, *.gradle, BUILD*\n");

    println!("NEOVIM SETUP:");
    println!("  Add to your LSP config:");
    println!("  require('lspconfig').bazbom.setup{{");
    println!("    cmd = {{ 'bazbom-lsp' }},");
    println!("    filetypes = {{ 'xml', 'groovy', 'kotlin', 'bzl' }},");
    println!("  }}\n");

    println!("FEATURES:");
    println!("  - Real-time vulnerability diagnostics in build files");
    println!("  - Quick fixes to upgrade vulnerable dependencies");
    println!("  - Hover information with CVE details");
    println!("  - Supports: pom.xml, build.gradle, build.gradle.kts, BUILD, BUILD.bazel\n");

    println!("For more information, see: https://github.com/cboyd0319/BazBOM/blob/main/crates/bazbom-lsp/README.md");

    Ok(())
}
