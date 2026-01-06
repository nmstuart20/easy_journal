use crate::config::Config;
use crate::error::Result;
use crate::journal::{github, gitlab};

/// Fetch and merge GitHub + GitLab items
pub async fn merge_git_integrations(config: &Config) -> Result<Option<String>> {
    // Fetch both sources concurrently
    let github_task = github::fetch_github_items(&config.github_config);
    let gitlab_task = gitlab::fetch_gitlab_items(&config.gitlab_config);

    let (github_result, gitlab_result) = tokio::join!(github_task, gitlab_task);

    // Handle GitHub (non-blocking on error)
    let github_items = match github_result {
        Ok(Some(items)) => Some(items),
        Ok(None) => None,
        Err(e) => {
            eprintln!("Warning: Could not fetch GitHub items: {}", e);
            None
        }
    };

    // Handle GitLab (non-blocking on error)
    let gitlab_items = match gitlab_result {
        Ok(Some(items)) => Some(items),
        Ok(None) => None,
        Err(e) => {
            eprintln!("Warning: Could not fetch GitLab items: {}", e);
            None
        }
    };

    // Merge results with section headers
    match (github_items, gitlab_items) {
        (Some(gh), Some(gl)) => Ok(Some(format!("### GitHub\n{}\n\n### GitLab\n{}", gh, gl))),
        (Some(gh), None) => Ok(Some(format!("### GitHub\n{}", gh))),
        (None, Some(gl)) => Ok(Some(format!("### GitLab\n{}", gl))),
        (None, None) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_merge_both_disabled() {
        let mut config = Config::default();
        config.github_config.enabled = false;
        config.gitlab_config.enabled = false;

        let result = merge_git_integrations(&config).await.unwrap();
        assert_eq!(result, None);
    }
}
