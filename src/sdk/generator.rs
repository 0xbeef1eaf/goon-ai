use crate::sdk::{analysis, metadata};
use std::path::Path;

pub fn generate_definitions(allowed_modules: &[String]) -> String {
    let all_modules = metadata::get_modules();
    let mut definitions = String::new();

    definitions.push_str("/** GoonAI SDK */\n");

    for module in &all_modules {
        let include = match module.permission {
            None => true, // Always include
            Some(perm) => {
                allowed_modules.contains(&perm.to_string())
                    || allowed_modules.contains(&"all".to_string())
            }
        };

        if include {
            // Analyze source file for ops if it exists
            let source_path = format!("src/sdk/{}.rs", module.name);
            let mut template = module.template.clone();

            if Path::new(&source_path).exists() {
                let (ops, structs) = analysis::analyze_source(Path::new(&source_path));
                for _op in ops {
                    // This is where we could auto-generate the function signature
                    // For now, we rely on the template, but we could verify or append here
                    // definitions.push_str(&format!("// Found op: {}\n", op.name));
                }

                // Inject struct and field documentation
                for info in structs {
                    // Inject struct docs
                    if !info.docs.is_empty() {
                        let doc_block = info
                            .docs
                            .iter()
                            .map(|d| format!(" * {}", d))
                            .collect::<Vec<_>>()
                            .join("\n");
                        let doc_comment = format!("/**\n{}\n */\n", doc_block);

                        let patterns = [
                            format!("interface {} ", info.name),
                            format!("type {} =", info.name),
                        ];

                        for pattern in patterns {
                            if let Some(idx) = template.find(&pattern) {
                                template.insert_str(idx, &doc_comment);
                                break;
                            }
                        }
                    }

                    // Inject field docs
                    for field in &info.fields {
                        if !field.docs.is_empty() {
                            let doc_block = field
                                .docs
                                .iter()
                                .map(|d| format!(" * {}", d))
                                .collect::<Vec<_>>()
                                .join("\n");
                            // Indent the doc comment to match field indentation (assuming 2 spaces)
                            let doc_comment = format!("\n  /**\n  {}\n   */", doc_block);

                            // Find the interface start again because indices shifted
                            let patterns = [
                                format!("interface {} ", info.name),
                                format!("type {} =", info.name),
                            ];

                            for pattern in patterns {
                                if let Some(interface_idx) = template.find(&pattern) {
                                    // Search for the field within this interface
                                    // We look for " field:" or " field?:" or " field :"
                                    // We limit search to some reasonable range or just next occurrence
                                    // To be safer, we could look for the field name preceded by whitespace and followed by ? or :

                                    // Simple heuristic: search for "  field:" or "  field?:" after interface_idx
                                    // We try both exact name and camelCase (simple lowercase first char)
                                    let field_names = vec![field.name.clone(), {
                                        let mut c = field.name.chars();
                                        match c.next() {
                                            None => String::new(),
                                            Some(f) => {
                                                f.to_lowercase().collect::<String>() + c.as_str()
                                            }
                                        }
                                    }];

                                    let mut inserted = false;
                                    for name in field_names {
                                        if name.is_empty() {
                                            continue;
                                        }
                                        // Patterns to match field declaration
                                        let field_patterns =
                                            [format!("  {}:", name), format!("  {}?:", name)];

                                        for field_pattern in field_patterns {
                                            // Search after interface definition
                                            if let Some(rel_idx) =
                                                template[interface_idx..].find(&field_pattern)
                                            {
                                                let abs_idx = interface_idx + rel_idx;
                                                // Insert before the newline that precedes the field?
                                                // Actually, ts-rs output is like:
                                                // interface Foo {
                                                //   field: type,
                                                // }
                                                // So we want to insert before "  field:"
                                                // But "  field:" match starts at the spaces.
                                                // So inserting there puts it on the same line?
                                                // No, we want it on previous line.
                                                // But we don't know if there is a newline before it (there should be).

                                                // Let's just insert at abs_idx.
                                                // doc_comment starts with \n.
                                                template.insert_str(abs_idx, &doc_comment);
                                                inserted = true;
                                                break;
                                            }
                                        }
                                        if inserted {
                                            break;
                                        }
                                    }
                                    if inserted {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            definitions.push_str(&format!("\n// Module: {}\n", module.name));
            definitions.push_str(&template);
            definitions.push('\n');
        }
    }

    // Generate global 'goon' namespace
    definitions.push_str("\ndeclare const goon: {\n");
    for module in all_modules {
        let include = match module.permission {
            None => true,
            Some(perm) => {
                allowed_modules.contains(&perm.to_string())
                    || allowed_modules.contains(&"all".to_string())
            }
        };

        if include {
            // We assume the class name matches the module name
            // e.g. module "image" -> class image
            // But we should check if the template actually exports it.
            // For now, we assume standard naming convention used in our TS files.
            // Some modules like "types" don't have a class to export on 'goon'.
            if module.name == "pack" {
                definitions.push_str("    pack: typeof Pack;\n");
            } else if module.name == "system" {
                definitions.push_str("    system: typeof System;\n");
            } else if module.name != "types" {
                definitions.push_str(&format!("    {}: typeof {};\n", module.name, module.name));
            }
        }
    }
    definitions.push_str("};\n");

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
