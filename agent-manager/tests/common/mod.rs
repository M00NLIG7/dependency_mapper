use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn create_temp_script(content: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("script.sh");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "#!/bin/bash").unwrap();
    write!(file, "{}", content).unwrap();
    file.flush().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    (dir, file_path)
}

pub fn create_temp_config(content: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("config.yaml");
    let mut file = File::create(&file_path).unwrap();
    write!(file, "{}", content).unwrap();
    file.flush().unwrap();
    (dir, file_path)
}
