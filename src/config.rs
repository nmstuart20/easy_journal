use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub journal_dir: PathBuf,
    pub template_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            journal_dir: PathBuf::from("journal"),
            template_path: PathBuf::from("template.md"),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
