use google_tasks1::TasksHub;
use google_tasks1::hyper;
use google_tasks1::hyper_rustls;

use crate::config::GoogleOAuthConfig;
use crate::error::{JournalError, Result};
use crate::journal::oauth;

/// Fetch all incomplete Google Tasks and format as markdown checkboxes
pub async fn fetch_google_tasks(oauth_config: &GoogleOAuthConfig) -> Result<Option<String>> {
    // Check if OAuth is configured
    let client_id = oauth_config.client_id.as_ref().ok_or_else(|| {
        JournalError::OAuthConfigMissing(
            "GOOGLE_CLIENT_ID not set. Run 'easy_journal auth google' first.".to_string(),
        )
    })?;

    let client_secret = oauth_config.client_secret.as_ref().ok_or_else(|| {
        JournalError::OAuthConfigMissing(
            "GOOGLE_CLIENT_SECRET not set. Run 'easy_journal auth google' first.".to_string(),
        )
    })?;

    // Check if token file exists
    if !oauth_config.token_storage_path.exists() {
        return Err(JournalError::OAuthConfigMissing(
            "No stored tokens found. Run 'easy_journal auth google' first.".to_string(),
        ));
    }

    // Load authenticator from stored tokens
    let auth = oauth::load_authenticator(
        client_id.clone(),
        client_secret.clone(),
        &oauth_config.token_storage_path,
    )
    .await?;

    // Create HTTP client (using hyper 0.14 from google-tasks1)
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .map_err(|e| {
            JournalError::GoogleTasksFailed(format!("Failed to load native roots: {}", e))
        })?
        .https_or_http()
        .enable_http1()
        .build();
    let client = hyper::Client::builder().build(https);

    // Create Tasks API hub
    let hub = TasksHub::new(client, auth);

    // Fetch all task lists
    let task_lists = hub
        .tasklists()
        .list()
        .doit()
        .await
        .map_err(|e| JournalError::GoogleTasksFailed(format!("Failed to fetch task lists: {}", e)))?
        .1;

    let mut all_tasks = Vec::new();

    // Fetch incomplete tasks from each list
    if let Some(items) = task_lists.items {
        for task_list in items {
            if let Some(task_list_id) = task_list.id {
                // Fetch tasks that are not completed
                let tasks_result = hub
                    .tasks()
                    .list(&task_list_id)
                    .show_completed(false) // Only fetch incomplete tasks
                    .doit()
                    .await;

                if let Ok((_, tasks_response)) = tasks_result {
                    if let Some(tasks) = tasks_response.items {
                        for task in tasks {
                            if let Some(title) = task.title {
                                // Only add tasks with non-empty titles
                                if !title.trim().is_empty() {
                                    all_tasks.push(title);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if all_tasks.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format_tasks(all_tasks)))
    }
}

/// Format tasks as markdown checkboxes
fn format_tasks(tasks: Vec<String>) -> String {
    tasks
        .iter()
        .map(|task| format!("- [ ] {}", task))
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tasks() {
        let tasks = vec![
            "Review pull request".to_string(),
            "Update documentation".to_string(),
            "Fix bug in authentication".to_string(),
        ];

        let formatted = format_tasks(tasks);

        assert_eq!(
            formatted,
            "- [ ] Review pull request\n- [ ] Update documentation\n- [ ] Fix bug in authentication"
        );
    }

    #[test]
    fn test_format_empty_tasks() {
        let tasks: Vec<String> = vec![];
        let formatted = format_tasks(tasks);
        assert_eq!(formatted, "");
    }
}
