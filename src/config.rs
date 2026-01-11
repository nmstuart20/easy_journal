use std::env;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub journal_dir: PathBuf,
    pub template_path: PathBuf,
    pub month_template_path: PathBuf,
    pub year_template_path: PathBuf,
    pub google_oauth: GoogleOAuthConfig,
    pub github_config: GitHubConfig,
    pub gitlab_config: GitLabConfig,
}

#[derive(Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub token_storage_path: PathBuf,
}

#[derive(Clone)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub enabled: bool,
}

#[derive(Clone)]
pub struct GitLabConfig {
    pub token: Option<String>,
    pub host: String,
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("."));
        let token_path = PathBuf::from(&home_dir).join(".easy_journal_tokens.json");

        Self {
            journal_dir: PathBuf::from("journal"),
            template_path: PathBuf::from("template.md"),
            month_template_path: PathBuf::from("month_template.md"),
            year_template_path: PathBuf::from("year_template.md"),
            google_oauth: GoogleOAuthConfig {
                client_id: env::var("GOOGLE_CLIENT_ID").ok(),
                client_secret: env::var("GOOGLE_CLIENT_SECRET").ok(),
                token_storage_path: token_path,
            },
            github_config: GitHubConfig {
                token: env::var("GITHUB_TOKEN").ok(),
                enabled: false,
            },
            gitlab_config: GitLabConfig {
                token: env::var("GITLAB_TOKEN").ok(),
                host: env::var("GITLAB_HOST").unwrap_or_else(|_| "https://gitlab.com".to_string()),
                enabled: false,
            },
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
