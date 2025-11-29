use crate::sdk::{analysis, metadata};
use std::path::Path;

pub fn generate_definitions(allowed_modules: &[String]) -> String {
    let all_modules = metadata::get_modules();
    let mut definitions = String::new();

    definitions.push_str("/** GoonAI SDK */\n");

    for module in all_modules {
        let include = match module.permission {
            None => true, // Always include
            Some(perm) => {
                allowed_modules.contains(&perm.to_string())
                    || allowed_modules.contains(&"all".to_string())
            }
        };

        if include {
            definitions.push_str(&format!("\n// Module: {}\n", module.name));
            definitions.push_str(&module.template);

            // Analyze source file for ops if it exists
            let source_path = format!("src/sdk/{}.rs", module.name);
            if Path::new(&source_path).exists() {
                let ops = analysis::analyze_source(Path::new(&source_path));
                for _op in ops {
                    // This is where we could auto-generate the function signature
                    // For now, we rely on the template, but we could verify or append here
                    // definitions.push_str(&format!("// Found op: {}\n", op.name));
                }
            }

            definitions.push('\n');
        }
    }

    definitions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_definitions_includes_always_modules() {
        let defs = generate_definitions(&[]);
        assert!(defs.contains("// Module: types"));
        assert!(defs.contains("// Module: pack"));
    }

    #[test]
    fn test_generate_definitions_includes_permitted_modules() {
        let defs = generate_definitions(&["image".to_string()]);
        assert!(defs.contains("// Module: image"));
        assert!(!defs.contains("// Module: video"));
    }

    #[test]
    fn test_generate_definitions_includes_all_modules() {
        let defs = generate_definitions(&["all".to_string()]);
        assert!(defs.contains("// Module: image"));
        assert!(defs.contains("// Module: video"));
        assert!(defs.contains("// Module: audio"));
    }
}
