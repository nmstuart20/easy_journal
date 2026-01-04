use std::env;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub journal_dir: PathBuf,
    pub template_path: PathBuf,
    pub google_oauth: GoogleOAuthConfig,
}

#[derive(Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub token_storage_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("."));
        let token_path = PathBuf::from(&home_dir).join(".easy_journal_tokens.json");

        Self {
            journal_dir: PathBuf::from("journal"),
            template_path: PathBuf::from("template.md"),
            google_oauth: GoogleOAuthConfig {
                client_id: env::var("GOOGLE_CLIENT_ID").ok(),
                client_secret: env::var("GOOGLE_CLIENT_SECRET").ok(),
                token_storage_path: token_path,
            },
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
