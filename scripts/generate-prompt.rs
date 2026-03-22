/// Build-time script to generate TypeScript prompt from canonical Rust source.
/// This ensures the web playground stays synchronized with the core implementation.

use repobox_core::prompt::REPOBOX_SYSTEM_PROMPT;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = Path::new("web/src/lib");
    
    // Ensure output directory exists
    fs::create_dir_all(output_dir)?;
    
    let ts_content = format!(
        "// GENERATED FILE - DO NOT EDIT MANUALLY\n\
         // Generated from repobox-core/src/prompt.rs by scripts/generate-prompt.rs\n\
         // To update this file, modify the canonical prompt and run: cargo run --bin generate-prompt\n\n\
         export const REPOBOX_SYSTEM_PROMPT = `{}`;\n",
        REPOBOX_SYSTEM_PROMPT
    );
    
    let output_path = output_dir.join("generated-prompt.ts");
    fs::write(&output_path, ts_content)?;
    
    println!("✓ Generated TypeScript prompt at {}", output_path.display());
    
    Ok(())
}