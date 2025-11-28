use crate::sdk::metadata;

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
            definitions.push_str(module.template);
            definitions.push('\n');
        }
    }

    definitions
}
