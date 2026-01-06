use crate::config::GitHubConfig;
use crate::error::{JournalError, Result};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct GitHubItem {
    pub title: String,
    pub url: String,
    pub number: u64,
    pub repo: String,
    pub labels: Vec<String>,
    pub due_date: Option<String>,
    pub item_type: GitHubItemType,
}

#[derive(Debug, Clone)]
pub enum GitHubItemType {
    AssignedIssue,
    CreatedIssue,
    AssignedPR,
    ReviewRequest,
}

#[derive(Deserialize, Debug)]
struct GitHubApiIssue {
    title: String,
    html_url: String,
    number: u64,
    repository_url: String,
    labels: Vec<GitHubApiLabel>,
    milestone: Option<GitHubApiMilestone>,
    pull_request: Option<serde_json::Value>, // Just check if exists
}

#[derive(Deserialize, Debug)]
struct GitHubApiLabel {
    name: String,
}

#[derive(Deserialize, Debug)]
struct GitHubApiMilestone {
    due_on: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GitHubSearchResponse {
    items: Vec<GitHubApiIssue>,
}

pub async fn fetch_github_items(config: &GitHubConfig) -> Result<Option<String>> {
    // Early return if not enabled
    if !config.enabled {
        return Ok(None);
    }

    let token = config.token.as_ref().ok_or_else(|| {
        JournalError::GitHubFailed(
            "GITHUB_TOKEN not set. Set the environment variable and use --github flag."
                .to_string(),
        )
    })?;

    // Build reqwest client
    let client = reqwest::Client::builder()
        .user_agent("easy_journal")
        .build()
        .map_err(|e| JournalError::GitHubFailed(format!("Failed to build HTTP client: {}", e)))?;

    // Fetch data concurrently
    let token_clone1 = token.clone();
    let client_clone1 = client.clone();
    let assigned_issues_task =
        tokio::task::spawn(async move { fetch_assigned_issues(&client_clone1, &token_clone1).await });

    let token_clone2 = token.clone();
    let client_clone2 = client.clone();
    let created_issues_task =
        tokio::task::spawn(async move { fetch_created_issues(&client_clone2, &token_clone2).await });

    let token_clone3 = token.clone();
    let client_clone3 = client.clone();
    let assigned_prs_task =
        tokio::task::spawn(async move { fetch_assigned_prs(&client_clone3, &token_clone3).await });

    let token_clone4 = token.clone();
    let client_clone4 = client.clone();
    let review_requests_task =
        tokio::task::spawn(async move { fetch_review_requests(&client_clone4, &token_clone4).await });

    let (assigned_issues, created_issues, assigned_prs, review_requests) = tokio::join!(
        assigned_issues_task,
        created_issues_task,
        assigned_prs_task,
        review_requests_task
    );

    // Unwrap the JoinHandle results
    let assigned_issues = assigned_issues
        .map_err(|e| JournalError::GitHubFailed(format!("Task join error: {}", e)))?;
    let created_issues = created_issues
        .map_err(|e| JournalError::GitHubFailed(format!("Task join error: {}", e)))?;
    let assigned_prs = assigned_prs
        .map_err(|e| JournalError::GitHubFailed(format!("Task join error: {}", e)))?;
    let review_requests = review_requests
        .map_err(|e| JournalError::GitHubFailed(format!("Task join error: {}", e)))?;

    // Combine all items (non-blocking on individual errors)
    let mut all_items = Vec::new();

    if let Ok(items) = assigned_issues {
        all_items.extend(items);
    }
    if let Ok(items) = created_issues {
        all_items.extend(items);
    }
    if let Ok(items) = assigned_prs {
        all_items.extend(items);
    }
    if let Ok(items) = review_requests {
        all_items.extend(items);
    }

    if all_items.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format_github_items(all_items)))
    }
}

async fn fetch_assigned_issues(
    client: &reqwest::Client,
    token: &str,
) -> Result<Vec<GitHubItem>> {
    let url = "https://api.github.com/issues";

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .query(&[("filter", "assigned"), ("state", "open"), ("per_page", "100")])
        .send()
        .await
        .map_err(|e| JournalError::GitHubFailed(format!("Failed to fetch assigned issues: {}", e)))?;

    let issues: Vec<GitHubApiIssue> = response.json().await.map_err(|e| {
        JournalError::GitHubFailed(format!("Failed to parse assigned issues: {}", e))
    })?;

    let items = issues
        .into_iter()
        .filter(|issue| issue.pull_request.is_none()) // Filter out PRs
        .map(|issue| {
            let repo = extract_repo_from_url(&issue.repository_url);
            let labels = issue.labels.iter().map(|l| l.name.clone()).collect();
            let due_date = issue
                .milestone
                .and_then(|m| m.due_on)
                .map(|d| d.split('T').next().unwrap_or(&d).to_string());

            GitHubItem {
                title: issue.title,
                url: issue.html_url,
                number: issue.number,
                repo,
                labels,
                due_date,
                item_type: GitHubItemType::AssignedIssue,
            }
        })
        .collect();

    Ok(items)
}

async fn fetch_created_issues(
    client: &reqwest::Client,
    token: &str,
) -> Result<Vec<GitHubItem>> {
    let url = "https://api.github.com/issues";

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .query(&[("filter", "created"), ("state", "open"), ("per_page", "100")])
        .send()
        .await
        .map_err(|e| JournalError::GitHubFailed(format!("Failed to fetch created issues: {}", e)))?;

    let issues: Vec<GitHubApiIssue> = response.json().await.map_err(|e| {
        JournalError::GitHubFailed(format!("Failed to parse created issues: {}", e))
    })?;

    let items = issues
        .into_iter()
        .filter(|issue| issue.pull_request.is_none()) // Filter out PRs
        .map(|issue| {
            let repo = extract_repo_from_url(&issue.repository_url);
            let labels = issue.labels.iter().map(|l| l.name.clone()).collect();
            let due_date = issue
                .milestone
                .and_then(|m| m.due_on)
                .map(|d| d.split('T').next().unwrap_or(&d).to_string());

            GitHubItem {
                title: issue.title,
                url: issue.html_url,
                number: issue.number,
                repo,
                labels,
                due_date,
                item_type: GitHubItemType::CreatedIssue,
            }
        })
        .collect();

    Ok(items)
}

async fn fetch_assigned_prs(
    client: &reqwest::Client,
    token: &str,
) -> Result<Vec<GitHubItem>> {
    let url = "https://api.github.com/issues";

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .query(&[("filter", "assigned"), ("state", "open"), ("per_page", "100")])
        .send()
        .await
        .map_err(|e| JournalError::GitHubFailed(format!("Failed to fetch assigned PRs: {}", e)))?;

    let issues: Vec<GitHubApiIssue> = response.json().await.map_err(|e| {
        JournalError::GitHubFailed(format!("Failed to parse assigned PRs: {}", e))
    })?;

    let items = issues
        .into_iter()
        .filter(|issue| issue.pull_request.is_some()) // Only include PRs
        .map(|issue| {
            let repo = extract_repo_from_url(&issue.repository_url);
            let labels = issue.labels.iter().map(|l| l.name.clone()).collect();
            let due_date = issue
                .milestone
                .and_then(|m| m.due_on)
                .map(|d| d.split('T').next().unwrap_or(&d).to_string());

            GitHubItem {
                title: issue.title,
                url: issue.html_url,
                number: issue.number,
                repo,
                labels,
                due_date,
                item_type: GitHubItemType::AssignedPR,
            }
        })
        .collect();

    Ok(items)
}

async fn fetch_review_requests(
    client: &reqwest::Client,
    token: &str,
) -> Result<Vec<GitHubItem>> {
    let url = "https://api.github.com/search/issues";
    let query = "type:pr state:open review-requested:@me";

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .query(&[("q", query), ("per_page", "100")])
        .send()
        .await
        .map_err(|e| {
            JournalError::GitHubFailed(format!("Failed to fetch review requests: {}", e))
        })?;

    let search_response: GitHubSearchResponse = response.json().await.map_err(|e| {
        JournalError::GitHubFailed(format!("Failed to parse review requests: {}", e))
    })?;

    let items = search_response
        .items
        .into_iter()
        .map(|issue| {
            let repo = extract_repo_from_url(&issue.repository_url);
            let labels = issue.labels.iter().map(|l| l.name.clone()).collect();
            let due_date = issue
                .milestone
                .and_then(|m| m.due_on)
                .map(|d| d.split('T').next().unwrap_or(&d).to_string());

            GitHubItem {
                title: issue.title,
                url: issue.html_url,
                number: issue.number,
                repo,
                labels,
                due_date,
                item_type: GitHubItemType::ReviewRequest,
            }
        })
        .collect();

    Ok(items)
}

fn extract_repo_from_url(url: &str) -> String {
    // Extract owner/repo from URL like "https://api.github.com/repos/owner/repo"
    url.trim_end_matches('/')
        .split('/')
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("/")
}

fn format_github_items(items: Vec<GitHubItem>) -> String {
    // Group by type
    let mut assigned_issues = Vec::new();
    let mut created_issues = Vec::new();
    let mut assigned_prs = Vec::new();
    let mut review_requests = Vec::new();

    for item in items {
        match item.item_type {
            GitHubItemType::AssignedIssue => assigned_issues.push(item),
            GitHubItemType::CreatedIssue => created_issues.push(item),
            GitHubItemType::AssignedPR => assigned_prs.push(item),
            GitHubItemType::ReviewRequest => review_requests.push(item),
        }
    }

    let mut sections = Vec::new();

    if !assigned_issues.is_empty() {
        sections.push(format_section("Assigned Issues", assigned_issues));
    }
    if !created_issues.is_empty() {
        sections.push(format_section("Created Issues", created_issues));
    }
    if !assigned_prs.is_empty() {
        sections.push(format_section("Assigned PRs", assigned_prs));
    }
    if !review_requests.is_empty() {
        sections.push(format_section("Review Requests", review_requests));
    }

    sections.join("\n\n")
}

fn format_section(title: &str, items: Vec<GitHubItem>) -> String {
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
            "- [ ] [{}] {} (#{}){}{}\n",
            item.repo, item.title, item.number, labels, due
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
    fn test_extract_repo_from_url() {
        let url = "https://api.github.com/repos/owner/repo";
        assert_eq!(extract_repo_from_url(url), "owner/repo");
    }

    #[test]
    fn test_format_github_items() {
        let items = vec![
            GitHubItem {
                title: "Fix bug".to_string(),
                url: "https://github.com/owner/repo/issues/123".to_string(),
                number: 123,
                repo: "owner/repo".to_string(),
                labels: vec!["bug".to_string(), "urgent".to_string()],
                due_date: Some("2026-01-15".to_string()),
                item_type: GitHubItemType::AssignedIssue,
            },
            GitHubItem {
                title: "Add feature".to_string(),
                url: "https://github.com/owner/repo/pull/456".to_string(),
                number: 456,
                repo: "owner/repo".to_string(),
                labels: vec![],
                due_date: None,
                item_type: GitHubItemType::ReviewRequest,
            },
        ];

        let output = format_github_items(items);
        assert!(output.contains("#### Assigned Issues"));
        assert!(output.contains("#### Review Requests"));
        assert!(output.contains("[bug] [urgent]"));
        assert!(output.contains("Due: 2026-01-15"));
        assert!(output.contains("Fix bug (#123)"));
        assert!(output.contains("Add feature (#456)"));
    }

    #[test]
    fn test_format_section() {
        let items = vec![GitHubItem {
            title: "Test issue".to_string(),
            url: "https://github.com/owner/repo/issues/1".to_string(),
            number: 1,
            repo: "owner/repo".to_string(),
            labels: vec!["test".to_string()],
            due_date: None,
            item_type: GitHubItemType::AssignedIssue,
        }];

        let output = format_section("Test Section", items);
        assert!(output.contains("#### Test Section"));
        assert!(output.contains("- [ ] [owner/repo] Test issue (#1) [test]"));
        assert!(output.contains("      https://github.com/owner/repo/issues/1"));
    }
}
