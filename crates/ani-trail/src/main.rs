slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let main_window = MainWindow::new()?;
    main_window.run()?;
    Ok(())
}
