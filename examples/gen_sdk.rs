use goon_ai::sdk::generate_typescript_definitions;
use std::fs;

fn main() {
    let definitions = generate_typescript_definitions(&["all".to_string()]);
    fs::write("/tmp/goon_sdk_output.ts", &definitions).expect("Failed to write SDK");
    println!("SDK written to /tmp/goon_sdk_output.ts");
}
