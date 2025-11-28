use crate::sdk;

pub struct SdkGenerator;

impl SdkGenerator {
    pub fn generate_definitions(allowed_modules: &[String]) -> String {
        sdk::generate_typescript_definitions(allowed_modules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_delegation() {
        let modules = vec!["image".to_string()];
        let defs = SdkGenerator::generate_definitions(&modules);
        assert!(defs.contains("class image"));
    }
}
