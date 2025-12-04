fn main() {
    // Compile all Slint UI components from a single entry point
    slint_build::compile_with_config(
        "src/ui/main.slint",
        slint_build::CompilerConfiguration::new()
            .with_include_paths(vec![std::path::PathBuf::from("src/media")]),
    )
    .unwrap();
}
