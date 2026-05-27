//! GitHub API integration tool
//!
//! Provides issue/PR/repo management via GitHub REST API.
//! Uses reqwest for HTTP calls with optional GitHub token authentication.

use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{ToolCategory, ToolManifest, ToolPermission};
use serde::Deserialize;

pub struct GitHubTool {
    token: Option<String>,
}

impl GitHubTool {
    pub fn new() -> Self {
        Self { token: None }
    }

    pub fn with_token(token: String) -> Self {
        Self { token: Some(token) }
    }

    /// Generate the ToolManifest for this tool
    pub fn manifest() -> ToolManifest {
        ToolManifest {
            id: "github".into(),
            name: "GitHub".into(),
            description: "Manage GitHub issues and pull requests via the GitHub REST API.".into(),
            long_description: Some(
                "List issues, pull requests, and repositories. Requires a GitHub personal \
                 access token for private repos."
                    .into(),
            ),
            category: ToolCategory::Developer,
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list_issues", "list_repos", "get_issue"],
                        "description": "GitHub operation to perform"
                    },
                    "owner": {
                        "type": "string",
                        "description": "GitHub repository owner (username or organization)"
                    },
                    "repo": {
                        "type": "string",
                        "description": "Repository name"
                    },
                    "state": {
                        "type": "string",
                        "enum": ["open", "closed", "all"],
                        "description": "Filter issues by state (default: open)"
                    },
                    "issue_number": {
                        "type": "integer",
                        "description": "Issue number (for get_issue action)"
                    }
                },
                "required": ["action", "owner", "repo"]
            }),
            permissions: vec![ToolPermission::NetworkAccess, ToolPermission::UserData],
            rate_limit: Some(10),
            timeout_ms: 30000,
            enabled_by_default: false,
            oauth: None,
            version: Some("1.0.0".into()),
            author: Some("FairyField".into()),
        }
    }

    /// Build the GitHub API URL
    fn api_url(owner: &str, repo: &str, path: &str) -> String {
        format!("https://api.github.com/repos/{}/{}{}", owner, repo, path)
    }

    /// Create an HTTP client with authorization header if token is set
    fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }

    /// Add authorization header to a request builder
    fn add_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref token) = self.token {
            request
                .header("Authorization", format!("Bearer {}", token))
                .header("User-Agent", "FairyField/1.0.0")
        } else {
            request.header("User-Agent", "FairyField/1.0.0")
        }
    }

    /// List issues for a repository
    pub async fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
    ) -> Result<String, String> {
        let url = Self::api_url(owner, repo, "/issues");
        let client = self.client();

        let mut request = client
            .get(&url)
            .query(&[("state", state), ("per_page", "30")]);
        request = self.add_auth(request);

        let response = request
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("GitHub API error ({status}): {body}",));
        }

        // Parse and simplify the response
        let issues: Vec<serde_json::Value> =
            serde_json::from_str(&body).map_err(|e| format!("Failed to parse response: {}", e))?;

        let simplified: Vec<serde_json::Value> = issues
            .iter()
            .map(|issue| {
                serde_json::json!({
                    "number": issue["number"],
                    "title": issue["title"],
                    "state": issue["state"],
                    "html_url": issue["html_url"],
                    "user": issue["user"]["login"],
                    "labels": issue["labels"].as_array().map(|arr| {
                        arr.iter().map(|l| &l["name"]).collect::<Vec<_>>()
                    }),
                    "created_at": issue["created_at"],
                    "updated_at": issue["updated_at"],
                    "comments": issue["comments"],
                })
            })
            .collect();

        serde_json::to_string(&simplified).map_err(|e| format!("Failed to serialize result: {}", e))
    }

    /// List repositories for a user or organization
    pub async fn list_repos(&self, owner: &str) -> Result<String, String> {
        let url = format!("https://api.github.com/users/{}/repos", owner);
        let client = self.client();

        let mut request = client
            .get(&url)
            .query(&[("per_page", "30"), ("sort", "updated")]);
        request = self.add_auth(request);

        let response = request
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("GitHub API error ({status}): {body}",));
        }

        let repos: Vec<serde_json::Value> =
            serde_json::from_str(&body).map_err(|e| format!("Failed to parse response: {}", e))?;

        let simplified: Vec<serde_json::Value> = repos
            .iter()
            .map(|repo| {
                serde_json::json!({
                    "name": repo["name"],
                    "full_name": repo["full_name"],
                    "description": repo["description"],
                    "html_url": repo["html_url"],
                    "language": repo["language"],
                    "stargazers_count": repo["stargazers_count"],
                    "fork": repo["fork"],
                    "updated_at": repo["updated_at"],
                })
            })
            .collect();

        serde_json::to_string(&simplified).map_err(|e| format!("Failed to serialize result: {}", e))
    }

    /// Get a single issue by number
    pub async fn get_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<String, String> {
        let url = Self::api_url(owner, repo, &format!("/issues/{}", issue_number));
        let client = self.client();

        let mut request = client.get(&url);
        request = self.add_auth(request);

        let response = request
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("GitHub API error ({status}): {body}",));
        }

        let issue: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| format!("Failed to parse response: {}", e))?;

        let simplified = serde_json::json!({
            "number": issue["number"],
            "title": issue["title"],
            "state": issue["state"],
            "body": issue["body"],
            "html_url": issue["html_url"],
            "user": issue["user"]["login"],
            "labels": issue["labels"].as_array().map(|arr| {
                arr.iter().map(|l| &l["name"]).collect::<Vec<_>>()
            }),
            "created_at": issue["created_at"],
            "updated_at": issue["updated_at"],
            "comments": issue["comments"],
        });

        serde_json::to_string(&simplified).map_err(|e| format!("Failed to serialize result: {}", e))
    }
}

impl Default for GitHubTool {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Tool trait implementation ----

#[derive(Deserialize)]
struct GitHubInput {
    action: String,
    owner: String,
    repo: String,
    #[serde(default = "default_state")]
    state: String,
    #[serde(default)]
    issue_number: Option<u64>,
}

fn default_state() -> String {
    "open".to_string()
}

impl Tool for GitHubTool {
    fn name(&self) -> &str {
        "github"
    }

    fn description(&self) -> &str {
        "Manage GitHub issues and pull requests"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list_issues", "list_repos", "get_issue"],
                    "description": "GitHub operation to perform"
                },
                "owner": {
                    "type": "string",
                    "description": "GitHub repository owner (username or organization)"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "state": {
                    "type": "string",
                    "enum": ["open", "closed", "all"],
                    "description": "Filter issues by state (default: open)"
                },
                "issue_number": {
                    "type": "integer",
                    "description": "Issue number (for get_issue action)"
                }
            },
            "required": ["action", "owner", "repo"]
        })
    }

    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<GitHubInput>(input).is_ok()
    }

    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let params = match serde_json::from_str::<GitHubInput>(input) {
            Ok(p) => p,
            Err(e) => {
                return Box::pin(async move { Err(ToolError::ValidationFailed(e.to_string())) })
            }
        };

        // We need owned values for the async block
        let token = self.token.clone();
        let action = params.action;
        let owner = params.owner;
        let repo = params.repo;
        let state = params.state;
        let issue_number = params.issue_number;

        Box::pin(async move {
            let tool = GitHubTool { token };

            match action.as_str() {
                "list_issues" => tool
                    .list_issues(&owner, &repo, &state)
                    .await
                    .map_err(ToolError::ExecutionFailed),
                "list_repos" => tool
                    .list_repos(&owner)
                    .await
                    .map_err(ToolError::ExecutionFailed),
                "get_issue" => {
                    let num = issue_number.ok_or_else(|| {
                        ToolError::ValidationFailed(
                            "issue_number is required for get_issue action".into(),
                        )
                    })?;
                    tool.get_issue(&owner, &repo, num)
                        .await
                        .map_err(ToolError::ExecutionFailed)
                }
                other => Err(ToolError::ValidationFailed(format!(
                    "Unknown action: {}",
                    other
                ))),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_input_valid_issues() {
        let tool = GitHubTool::new();
        assert!(tool.validate_input(
            r#"{"action": "list_issues", "owner": "testowner", "repo": "testrepo"}"#
        ));
    }

    #[test]
    fn validate_input_valid_repos() {
        let tool = GitHubTool::new();
        assert!(tool.validate_input(
            r#"{"action": "list_repos", "owner": "testowner", "repo": "testrepo"}"#
        ));
    }

    #[test]
    fn validate_input_valid_get_issue() {
        let tool = GitHubTool::new();
        assert!(tool.validate_input(
            r#"{"action": "get_issue", "owner": "testowner", "repo": "testrepo", "issue_number": 42}"#
        ));
    }

    #[test]
    fn validate_input_invalid_json() {
        let tool = GitHubTool::new();
        assert!(!tool.validate_input("not json"));
    }

    #[test]
    fn validate_input_missing_required() {
        let tool = GitHubTool::new();
        assert!(!tool.validate_input(r#"{"action": "list_issues"}"#));
    }

    #[test]
    fn validate_input_default_state() {
        let tool = GitHubTool::new();
        assert!(tool.validate_input(r#"{"action": "list_issues", "owner": "o", "repo": "r"}"#));
    }

    #[test]
    fn name_and_description() {
        let tool = GitHubTool::new();
        assert_eq!(tool.name(), "github");
        assert_eq!(tool.description(), "Manage GitHub issues and pull requests");
    }

    #[test]
    fn parameters_schema_has_required_fields() {
        let tool = GitHubTool::new();
        let schema = tool.parameters_schema();
        assert!(schema.is_object());
        let required = schema["required"].as_array().unwrap();
        let required_strs: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(required_strs.contains(&"action"));
        assert!(required_strs.contains(&"owner"));
        assert!(required_strs.contains(&"repo"));
    }

    #[test]
    fn manifest_has_correct_id() {
        let manifest = GitHubTool::manifest();
        assert_eq!(manifest.id, "github");
        assert_eq!(manifest.category, ToolCategory::Developer);
        assert!(!manifest.enabled_by_default);
        // verify permissions include NetworkAccess
        assert!(manifest
            .permissions
            .contains(&ToolPermission::NetworkAccess));
    }

    #[tokio::test]
    async fn execute_unknown_action_errors() {
        let tool = GitHubTool::new();
        let result = tool
            .execute(r#"{"action": "unknown_action", "owner": "o", "repo": "r"}"#)
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ToolError::ValidationFailed(msg) => {
                assert!(msg.contains("Unknown action"));
            }
            _ => panic!("Expected ValidationFailed, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn execute_get_issue_missing_number_errors() {
        let tool = GitHubTool::new();
        let result = tool
            .execute(r#"{"action": "get_issue", "owner": "o", "repo": "r"}"#)
            .await;
        assert!(result.is_err());
    }
}
