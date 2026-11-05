fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(tela_ui::import_paths())
        .with_style("fluent".into());
    slint_build::compile_with_config("ui/MainWindow.slint", config).unwrap();
}
