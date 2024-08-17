use std::io::Write;
use tempfile::NamedTempFile;

pub fn create_temp_config_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file
}
