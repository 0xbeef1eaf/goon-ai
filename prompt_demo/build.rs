fn main() {
    slint_build::compile_with_config(
        "gallery.slint",
        slint_build::CompilerConfiguration::new()
            .with_include_paths(vec![std::path::PathBuf::from(".")]),
    )
    .unwrap();
}
