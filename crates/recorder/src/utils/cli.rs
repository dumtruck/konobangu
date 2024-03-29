pub fn hack_env_to_fit_workspace() -> std::io::Result<()> {
    if cfg!(test) || cfg!(debug_assertions) {
        let package_dir = env!("CARGO_MANIFEST_DIR");
        let package_dir = std::path::Path::new(package_dir);
        std::env::set_current_dir(package_dir)?;
    }
    Ok(())
}
