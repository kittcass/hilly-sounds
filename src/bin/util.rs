use std::path::PathBuf;

pub fn validate_is_file(arg: &str) -> Result<(), String> {
    if !PathBuf::from(arg).is_file() {
        Err(String::from("not a file"))
    } else {
        Ok(())
    }
}
