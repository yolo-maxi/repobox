use std::collections::HashMap;

// Import the parser module items we need
use repobox::config::*;
use repobox::parser::parse;

fn main() {
    let yaml = r#"
permissions:
  rules:
    - "vitalik.eth push >main"
"#;

    match parse(yaml) {
        Ok(config) => {
            println!("Parse succeeded!");
            for rule in &config.permissions.rules {
                println!("Rule: {:?}", rule);
                match &rule.subject {
                    Subject::Identity(id) => println!("  Subject: Identity({:?})", id),
                    Subject::Group(name) => println!("  Subject: Group({})", name),
                    Subject::All => println!("  Subject: All"),
                }
            }
        }
        Err(e) => {
            println!("Parse failed: {}", e);
        }
    }
}