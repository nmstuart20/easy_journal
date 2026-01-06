use crate::config::GitLabConfig;
use crate::error::{JournalError, Result};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct GitLabItem {
    pub title: String,
    pub url: String,
    pub iid: u64,
    pub project: String,
    pub labels: Vec<String>,
    pub due_date: Option<String>,
    pub item_type: GitLabItemType,
}

#[derive(Debug, Clone)]
pub enum GitLabItemType {
    AssignedIssue,
    CreatedIssue,
    AssignedMR,
    ReviewRequest,
}

#[derive(Deserialize, Debug)]
struct GitLabApiIssue {
    title: String,
    web_url: String,
    iid: u64,
    labels: Vec<String>,
    due_date: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GitLabApiMR {
    title: String,
    web_url: String,
    iid: u64,
    labels: Vec<String>,
}

pub async fn fetch_gitlab_items(config: &GitLabConfig) -> Result<Option<String>> {
    // Early return if not enabled
    if !config.enabled {
        return Ok(None);
    }

    let token = config.token.as_ref().ok_or_else(|| {
        JournalError::GitLabFailed(
            "GITLAB_TOKEN not set. Set the environment variable and use --gitlab flag."
                .to_string(),
        )
    })?;

    // Build reqwest client
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| JournalError::GitLabFailed(format!("Failed to build HTTP client: {}", e)))?;

    // Fetch data concurrently using spawn_blocking for blocking operations
    let config_clone = config.clone();
    let token_clone = token.clone();
    let client_clone1 = client.clone();
    let assigned_issues_task = tokio::task::spawn(async move {
        fetch_assigned_issues(&client_clone1, &config_clone.host, &token_clone).await
    });

    let config_clone = config.clone();
    let token_clone = token.clone();
    let client_clone2 = client.clone();
    let created_issues_task = tokio::task::spawn(async move {
        fetch_created_issues(&client_clone2, &config_clone.host, &token_clone).await
    });

    let config_clone = config.clone();
    let token_clone = token.clone();
    let client_clone3 = client.clone();
    let assigned_mrs_task = tokio::task::spawn(async move {
        fetch_assigned_mrs(&client_clone3, &config_clone.host, &token_clone).await
    });

    let config_clone = config.clone();
    let token_clone = token.clone();
    let client_clone4 = client.clone();
    let review_requests_task = tokio::task::spawn(async move {
        fetch_review_requests(&client_clone4, &config_clone.host, &token_clone).await
    });

    let (assigned_issues, created_issues, assigned_mrs, review_requests) = tokio::join!(
        assigned_issues_task,
        created_issues_task,
        assigned_mrs_task,
        review_requests_task
    );

    // Unwrap the JoinHandle results
    let assigned_issues = assigned_issues
        .map_err(|e| JournalError::GitLabFailed(format!("Task join error: {}", e)))?;
    let created_issues = created_issues
        .map_err(|e| JournalError::GitLabFailed(format!("Task join error: {}", e)))?;
    let assigned_mrs = assigned_mrs
        .map_err(|e| JournalError::GitLabFailed(format!("Task join error: {}", e)))?;
    let review_requests = review_requests
        .map_err(|e| JournalError::GitLabFailed(format!("Task join error: {}", e)))?;

    // Combine all items (non-blocking on individual errors)
    let mut all_items = Vec::new();

    if let Ok(items) = assigned_issues {
        all_items.extend(items);
    }
    if let Ok(items) = created_issues {
        all_items.extend(items);
    }
    if let Ok(items) = assigned_mrs {
        all_items.extend(items);
    }
    if let Ok(items) = review_requests {
        all_items.extend(items);
    }

    if all_items.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format_gitlab_items(all_items)))
    }
}

async fn fetch_assigned_issues(
    client: &reqwest::Client,
    host: &str,
    token: &str,
) -> Result<Vec<GitLabItem>> {
    let url = format!("{}/api/v4/issues", host.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("PRIVATE-TOKEN", token)
        .query(&[("scope", "assigned_to_me"), ("state", "opened")])
        .send()
        .await
        .map_err(|e| JournalError::GitLabFailed(format!("Failed to fetch assigned issues: {}", e)))?;

    let issues: Vec<GitLabApiIssue> = response.json().await.map_err(|e| {
        JournalError::GitLabFailed(format!("Failed to parse assigned issues: {}", e))
    })?;

    let items = issues
        .into_iter()
        .map(|issue| {
            let project = extract_project_from_url(&issue.web_url);
            GitLabItem {
                title: issue.title,
                url: issue.web_url,
                iid: issue.iid,
                project,
                labels: issue.labels,
                due_date: issue.due_date,
                item_type: GitLabItemType::AssignedIssue,
            }
        })
        .collect();

    Ok(items)
}

async fn fetch_created_issues(
    client: &reqwest::Client,
    host: &str,
    token: &str,
) -> Result<Vec<GitLabItem>> {
    let url = format!("{}/api/v4/issues", host.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("PRIVATE-TOKEN", token)
        .query(&[("scope", "created_by_me"), ("state", "opened")])
        .send()
        .await
        .map_err(|e| JournalError::GitLabFailed(format!("Failed to fetch created issues: {}", e)))?;

    let issues: Vec<GitLabApiIssue> = response.json().await.map_err(|e| {
        JournalError::GitLabFailed(format!("Failed to parse created issues: {}", e))
    })?;

    let items = issues
        .into_iter()
        .map(|issue| {
            let project = extract_project_from_url(&issue.web_url);
            GitLabItem {
                title: issue.title,
                url: issue.web_url,
                iid: issue.iid,
                project,
                labels: issue.labels,
                due_date: issue.due_date,
                item_type: GitLabItemType::CreatedIssue,
            }
        })
        .collect();

    Ok(items)
}

async fn fetch_assigned_mrs(
    client: &reqwest::Client,
    host: &str,
    token: &str,
) -> Result<Vec<GitLabItem>> {
    let url = format!("{}/api/v4/merge_requests", host.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("PRIVATE-TOKEN", token)
        .query(&[("scope", "assigned_to_me"), ("state", "opened")])
        .send()
        .await
        .map_err(|e| JournalError::GitLabFailed(format!("Failed to fetch assigned MRs: {}", e)))?;

    let mrs: Vec<GitLabApiMR> = response
        .json()
        .await
        .map_err(|e| JournalError::GitLabFailed(format!("Failed to parse assigned MRs: {}", e)))?;

    let items = mrs
        .into_iter()
        .map(|mr| {
            let project = extract_project_from_url(&mr.web_url);
            GitLabItem {
                title: mr.title,
                url: mr.web_url,
                iid: mr.iid,
                project,
                labels: mr.labels,
                due_date: None,
                item_type: GitLabItemType::AssignedMR,
            }
        })
        .collect();

    Ok(items)
}

async fn fetch_review_requests(
    client: &reqwest::Client,
    host: &str,
    token: &str,
) -> Result<Vec<GitLabItem>> {
    let url = format!("{}/api/v4/merge_requests", host.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("PRIVATE-TOKEN", token)
        .query(&[("reviewer_id", "me"), ("state", "opened")])
        .send()
        .await
        .map_err(|e| {
            JournalError::GitLabFailed(format!("Failed to fetch review requests: {}", e))
        })?;

    let mrs: Vec<GitLabApiMR> = response.json().await.map_err(|e| {
        JournalError::GitLabFailed(format!("Failed to parse review requests: {}", e))
    })?;

    let items = mrs
        .into_iter()
        .map(|mr| {
            let project = extract_project_from_url(&mr.web_url);
            GitLabItem {
                title: mr.title,
                url: mr.web_url,
                iid: mr.iid,
                project,
                labels: mr.labels,
                due_date: None,
                item_type: GitLabItemType::ReviewRequest,
            }
        })
        .collect();

    Ok(items)
}

fn extract_project_from_url(url: &str) -> String {
    // Extract group/project from URL like "https://gitlab.com/group/project/-/issues/123"
    // or "https://gitlab.com/group/subgroup/project/-/merge_requests/456"
    let parts: Vec<&str> = url.split('/').collect();
    if let Some(dash_idx) = parts.iter().position(|&p| p == "-") {
        if dash_idx >= 2 {
            // Join all parts between the domain and the "/-/" separator
            let start = parts
                .iter()
                .position(|&p| p.contains('.'))
                .map(|i| i + 1)
                .unwrap_or(3);
            return parts[start..dash_idx].join("/");
        }
    }
    "unknown".to_string()
}

fn format_gitlab_items(items: Vec<GitLabItem>) -> String {
    // Group by type
    let mut assigned_issues = Vec::new();
    let mut created_issues = Vec::new();
    let mut assigned_mrs = Vec::new();
    let mut review_requests = Vec::new();

    for item in items {
        match item.item_type {
            GitLabItemType::AssignedIssue => assigned_issues.push(item),
            GitLabItemType::CreatedIssue => created_issues.push(item),
            GitLabItemType::AssignedMR => assigned_mrs.push(item),
            GitLabItemType::ReviewRequest => review_requests.push(item),
        }
    }

    let mut sections = Vec::new();

    if !assigned_issues.is_empty() {
        sections.push(format_section("Assigned Issues", assigned_issues));
    }
    if !created_issues.is_empty() {
        sections.push(format_section("Created Issues", created_issues));
    }
    if !assigned_mrs.is_empty() {
        sections.push(format_section("Assigned MRs", assigned_mrs));
    }
    if !review_requests.is_empty() {
        sections.push(format_section("Review Requests", review_requests));
    }

    sections.join("\n\n")
}

fn format_section(title: &str, items: Vec<GitLabItem>) -> String {
    let mut output = format!("#### {}\n", title);

    for item in items {
        // Format labels
        let labels = if item.labels.is_empty() {
            String::new()
        } else {
            format!(" [{}]", item.labels.join("] ["))
        };

        // Format due date
        let due = item
            .due_date
            .map(|d| format!(" - Due: {}", d))
            .unwrap_or_default();

        // Main line
        output.push_str(&format!(
            "- [ ] [{}] {} (!{}){}{}\n",
            item.project, item.title, item.iid, labels, due
        ));

        // URL on second line (indented)
        output.push_str(&format!("      {}\n", item.url));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_project_from_url() {
        let url = "https://gitlab.com/group/project/-/issues/123";
        assert_eq!(extract_project_from_url(url), "group/project");

        let url2 = "https://gitlab.com/group/subgroup/project/-/merge_requests/456";
        assert_eq!(extract_project_from_url(url2), "group/subgroup/project");
    }

    #[test]
    fn test_format_gitlab_items() {
        let items = vec![
            GitLabItem {
                title: "Fix bug".to_string(),
                url: "https://gitlab.com/group/project/-/issues/123".to_string(),
                iid: 123,
                project: "group/project".to_string(),
                labels: vec!["bug".to_string(), "urgent".to_string()],
                due_date: Some("2026-01-15".to_string()),
                item_type: GitLabItemType::AssignedIssue,
            },
            GitLabItem {
                title: "Add feature".to_string(),
                url: "https://gitlab.com/group/project/-/merge_requests/456".to_string(),
                iid: 456,
                project: "group/project".to_string(),
                labels: vec![],
                due_date: None,
                item_type: GitLabItemType::ReviewRequest,
            },
        ];

        let output = format_gitlab_items(items);
        assert!(output.contains("#### Assigned Issues"));
        assert!(output.contains("#### Review Requests"));
        assert!(output.contains("[bug] [urgent]"));
        assert!(output.contains("Due: 2026-01-15"));
        assert!(output.contains("Fix bug (!123)"));
        assert!(output.contains("Add feature (!456)"));
    }

    #[test]
    fn test_format_section() {
        let items = vec![GitLabItem {
            title: "Test issue".to_string(),
            url: "https://gitlab.com/group/project/-/issues/1".to_string(),
            iid: 1,
            project: "group/project".to_string(),
            labels: vec!["test".to_string()],
            due_date: None,
            item_type: GitLabItemType::AssignedIssue,
        }];

        let output = format_section("Test Section", items);
        assert!(output.contains("#### Test Section"));
        assert!(output.contains("- [ ] [group/project] Test issue (!1) [test]"));
        assert!(output.contains("      https://gitlab.com/group/project/-/issues/1"));
    }
}
