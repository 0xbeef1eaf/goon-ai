use crate::sdk::{analysis, metadata};
use std::path::Path;

/// Convert a Rust op function name to a TypeScript method name.
/// e.g., "op_show_image" -> "show"
/// e.g., "op_play_audio" -> "play"
/// e.g., "op_set_mood" -> "setMood"
fn op_name_to_ts_method(op_name: &str) -> String {
    // Remove "op_" prefix
    let name = op_name.strip_prefix("op_").unwrap_or(op_name);

    // Split by underscores and convert to camelCase
    let parts: Vec<&str> = name.split('_').collect();
    if parts.is_empty() {
        return name.to_string();
    }

    // First part stays lowercase, rest get capitalized first letter
    let mut result = parts[0].to_string();
    for part in &parts[1..] {
        if !part.is_empty() {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                result.push_str(&first.to_uppercase().to_string());
                result.push_str(chars.as_str());
            }
        }
    }

    result
}

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

                // Auto-generate function signatures from ops
                for op in &ops {
                    let ts_method_name = op_name_to_ts_method(&op.name);

                    // Generate doc comment if docs exist
                    if !op.docs.is_empty() {
                        let doc_block = op
                            .docs
                            .iter()
                            .map(|d| format!("   * {}", d))
                            .collect::<Vec<_>>()
                            .join("\n");

                        // Find the method in the template and inject docs before it
                        let method_patterns = [
                            format!("{}(", ts_method_name),
                            format!("{} (", ts_method_name),
                        ];

                        for pattern in method_patterns {
                            if let Some(idx) = template.find(&pattern) {
                                // Look back a reasonable amount to check for existing JSDoc
                                // (JSDoc comments are typically within 20 lines / ~500 chars before the method)
                                let look_back_start = idx.saturating_sub(500);
                                let preceding_content = &template[look_back_start..idx];

                                // Check if there's a JSDoc comment that ends close to the method
                                // by looking for "*/" followed by mostly whitespace until the method
                                if let Some(jsdoc_end) = preceding_content.rfind("*/") {
                                    // Check if there's only whitespace and keywords between */ and the method
                                    let between = &preceding_content[jsdoc_end + 2..];
                                    let between_trimmed = between.trim();
                                    // Allow common method modifiers between JSDoc and method name
                                    let is_only_modifiers = between_trimmed.is_empty()
                                        || between_trimmed == "static"
                                        || between_trimmed == "static async"
                                        || between_trimmed == "async"
                                        || between_trimmed.starts_with("static");
                                    if is_only_modifiers {
                                        // Already has a JSDoc comment, skip injection
                                        break;
                                    }
                                }

                                // Find the start of the line (after previous newline)
                                let line_start =
                                    template[..idx].rfind('\n').map(|i| i + 1).unwrap_or(0);
                                let doc_comment = format!("  /**\n{}   */\n  ", doc_block);
                                template.insert_str(line_start, &doc_comment);
                                break;
                            }
                        }
                    }
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
    fn test_op_name_to_ts_method() {
        assert_eq!(op_name_to_ts_method("op_show_image"), "showImage");
        assert_eq!(op_name_to_ts_method("op_play_audio"), "playAudio");
        assert_eq!(op_name_to_ts_method("op_set_mood"), "setMood");
        assert_eq!(op_name_to_ts_method("op_show"), "show");
        assert_eq!(op_name_to_ts_method("show_image"), "showImage");
    }

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
