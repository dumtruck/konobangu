use std::path::Path;

pub fn load_test_env() -> Result<(), dotenv::Error> {
    let package_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let env_files = vec![
        package_dir.join("configs/test.local.env"),
        package_dir.join("configs/test.env"),
    ];
    for env_file in env_files {
        if env_file.exists() {
            dotenv::from_path(env_file)?;
            break;
        }
    }
    Ok(())
}

pub fn load_test_env_panic() {
    load_test_env().expect("failed to load test env")
}
