pub fn generate_typescript_definitions(allowed_modules: &[String]) -> String {
    let mut definitions = String::new();
    
    definitions.push_str("/** GoonAI SDK */\n");
    definitions.push_str("type WindowHandle = number;\n");
    
    if allowed_modules.contains(&"image".to_string()) {
         definitions.push_str(r#"
/**
 * Display an image on screen
 * @param path - Asset path or tag query
 * @param options - Display options
 */
declare function showImage(
    path: string,
    options?: {
      duration?: number;
      opacity?: number;
      position?: { x: number; y: number };
      alwaysOnTop?: boolean;
    }
): Promise<WindowHandle>;
"#);
    }
    
    definitions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_definitions_empty() {
        let modules = vec![];
        let defs = generate_typescript_definitions(&modules);
        assert!(defs.contains("/** GoonAI SDK */"));
        assert!(defs.contains("type WindowHandle = number;"));
        assert!(!defs.contains("declare function showImage"));
    }

    #[test]
    fn test_generate_definitions_image() {
        let modules = vec!["image".to_string()];
        let defs = generate_typescript_definitions(&modules);
        assert!(defs.contains("declare function showImage"));
    }
}
