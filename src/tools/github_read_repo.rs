//! GitHub connector — Read Repo tool.
//!
//! Reads files from an existing GitHub repository into the active sandbox.
//! Uses the GitHub REST API (git trees + blobs endpoints), the same pattern
//! as `github_push.rs`. No new dependencies required.
//!
//! Typical use: agent fetches an existing repo before applying a bug fix,
//! then pushes a new branch and opens a PR.

use super::traits::{Tool, ToolResult};
use crate::config::ZerobuildConfig;
use crate::sandbox::SandboxClient;
use crate::store;
use async_trait::async_trait;
use base64::Engine as _;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

const TOOL_NAME: &str = "github_read_repo";
const GITHUB_API_BASE: &str = "https://api.github.com";
const MAX_FILES: usize = 500;

/// File extensions treated as binary — skip these to avoid writing garbage.
const BINARY_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".svg", ".webp", ".bmp", ".tiff", ".avif", ".woff",
    ".woff2", ".ttf", ".otf", ".eot", ".pdf", ".zip", ".tar", ".gz", ".7z", ".rar", ".exe", ".bin",
    ".dll", ".so", ".dylib", ".wasm", ".lock",
];

/// Directory names to skip entirely.
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".next",
    ".nuxt",
    "target",
    "dist",
    "build",
    ".git",
    ".cache",
    "__pycache__",
    ".venv",
    "venv",
];

pub struct GitHubReadRepoTool {
    client: Arc<dyn SandboxClient>,
    config: Arc<ZerobuildConfig>,
}

impl GitHubReadRepoTool {
    pub fn new(client: Arc<dyn SandboxClient>, config: Arc<ZerobuildConfig>) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl Tool for GitHubReadRepoTool {
    fn name(&self) -> &str {
        TOOL_NAME
    }

    fn description(&self) -> &str {
        "Read all text files from an existing GitHub repository into the active sandbox. \
         Fetches the full file tree and writes each file under the specified workdir. \
         Use this before applying a bug fix to an existing repo. \
         Requires an active sandbox and GitHub authentication (use github_connect first). \
         Skips binary files and large dependency directories automatically."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "GitHub repository owner (user or org). Example: octocat"
                },
                "repo": {
                    "type": "string",
                    "description": "GitHub repository name. Example: my-app"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch to read from. Default: main"
                },
                "workdir": {
                    "type": "string",
                    "description": "Destination directory inside sandbox. Default: project"
                }
            },
            "required": ["owner", "repo"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 1. Require active sandbox
        if let Err(e) = self.client.require_id() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(e),
                error_hint: Some(
                    "Create a sandbox first with sandbox_create, then call github_read_repo."
                        .to_string(),
                ),
            });
        }

        let owner = match args["owner"].as_str().filter(|s| !s.is_empty()) {
            Some(v) => v.to_string(),
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing required parameter: owner".to_string()),
                    error_hint: None,
                })
            }
        };

        let repo = match args["repo"].as_str().filter(|s| !s.is_empty()) {
            Some(v) => v.to_string(),
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing required parameter: repo".to_string()),
                    error_hint: None,
                })
            }
        };

        let branch = args["branch"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("main")
            .to_string();

        let workdir = args["workdir"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("project")
            .to_string();

        // 2. Load GitHub token
        let db_path = PathBuf::from(&self.config.db_path);
        let conn = store::init_db(&db_path)
            .map_err(|e| anyhow::anyhow!("Failed to open store DB: {e}"))?;
        let tok = match store::tokens::load_github_token(&conn) {
            Ok(Some(t)) => t,
            Ok(None) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(
                        "GitHub is not connected. Use github_connect to authenticate first."
                            .to_string(),
                    ),
                    error_hint: None,
                })
            }
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to load GitHub token: {e}")),
                    error_hint: None,
                })
            }
        };
        let token = &tok.token;

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("ZeroBuild/0.1")
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {e}"))?;

        // 3. Fetch recursive git tree
        let tree_url =
            format!("{GITHUB_API_BASE}/repos/{owner}/{repo}/git/trees/{branch}?recursive=1");
        let tree_resp = http
            .get(&tree_url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("GitHub API request failed: {e}"))?;

        if !tree_resp.status().is_success() {
            let status = tree_resp.status();
            let body = tree_resp.text().await.unwrap_or_default();
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Failed to fetch repo tree ({status}): {body}"
                )),
                error_hint: Some(format!(
                    "Make sure the repo '{owner}/{repo}' exists and the branch '{branch}' is correct."
                )),
            });
        }

        let tree_data: serde_json::Value = tree_resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse tree response: {e}"))?;

        let entries = match tree_data["tree"].as_array() {
            Some(a) => a.clone(),
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Unexpected tree response format from GitHub API.".to_string()),
                    error_hint: None,
                })
            }
        };

        // Filter to blob entries only, skip ignored dirs and binary extensions
        let blobs: Vec<(String, String)> = entries
            .iter()
            .filter_map(|entry| {
                if entry["type"].as_str() != Some("blob") {
                    return None;
                }
                let path = entry["path"].as_str()?;
                let sha = entry["sha"].as_str()?;
                if should_skip(path) {
                    return None;
                }
                Some((path.to_string(), sha.to_string()))
            })
            .take(MAX_FILES)
            .collect();

        if blobs.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "No readable text files found in '{owner}/{repo}' on branch '{branch}'."
                )),
                error_hint: None,
            });
        }

        let total = blobs.len();
        let mut written = 0usize;
        let mut skipped = 0usize;

        // 4. Fetch each blob and write to sandbox
        for (path, sha) in &blobs {
            let blob_url = format!("{GITHUB_API_BASE}/repos/{owner}/{repo}/git/blobs/{sha}");

            let blob_resp = match http
                .get(&blob_url)
                .header("Authorization", format!("Bearer {token}"))
                .header("Accept", "application/vnd.github+json")
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!("github_read_repo: skip {path} — fetch error: {e}");
                    skipped += 1;
                    continue;
                }
            };

            if !blob_resp.status().is_success() {
                tracing::warn!(
                    "github_read_repo: skip {path} — blob API returned {}",
                    blob_resp.status()
                );
                skipped += 1;
                continue;
            }

            let blob_data: serde_json::Value = match blob_resp.json().await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!("github_read_repo: skip {path} — parse error: {e}");
                    skipped += 1;
                    continue;
                }
            };

            let encoded = match blob_data["content"].as_str() {
                Some(c) => c.replace('\n', ""),
                None => {
                    skipped += 1;
                    continue;
                }
            };

            let bytes = match base64::engine::general_purpose::STANDARD.decode(&encoded) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("github_read_repo: skip {path} — base64 decode error: {e}");
                    skipped += 1;
                    continue;
                }
            };

            let content = match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => {
                    tracing::warn!("github_read_repo: skip {path} — not valid UTF-8 (binary)");
                    skipped += 1;
                    continue;
                }
            };

            let dest = format!("{workdir}/{path}");
            match self.client.write_file(&dest, &content).await {
                Ok(()) => written += 1,
                Err(e) => {
                    tracing::warn!("github_read_repo: skip {path} — write_file error: {e}");
                    skipped += 1;
                }
            }
        }

        Ok(ToolResult {
            success: true,
            output: format!(
                "Read repo '{owner}/{repo}' (branch: {branch}) into sandbox '{workdir}/'.\n\
                 Files found: {total} | Written: {written} | Skipped: {skipped}"
            ),
            error: None,
            error_hint: None,
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns true if the file at `path` should be skipped.
fn should_skip(path: &str) -> bool {
    // Skip known large/binary directories
    for dir in SKIP_DIRS {
        if path.starts_with(&format!("{dir}/")) || path == *dir {
            return true;
        }
        // Handle nested: any segment matches
        if path.split('/').any(|segment| segment == *dir) {
            return true;
        }
    }

    // Skip binary file extensions
    let lower = path.to_lowercase();
    for ext in BINARY_EXTENSIONS {
        if lower.ends_with(ext) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ZerobuildConfig;
    use crate::sandbox::local::LocalProcessSandboxClient;
    use tempfile::TempDir;

    fn make_tool(tmp: &TempDir) -> GitHubReadRepoTool {
        let sandbox = Arc::new(LocalProcessSandboxClient::new());
        let config = Arc::new(ZerobuildConfig {
            db_path: tmp.path().join("test.db").to_string_lossy().to_string(),
            ..ZerobuildConfig::default()
        });
        GitHubReadRepoTool::new(sandbox, config)
    }

    #[test]
    fn tool_name() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(make_tool(&tmp).name(), TOOL_NAME);
    }

    #[test]
    fn tool_has_description() {
        let tmp = TempDir::new().unwrap();
        assert!(!make_tool(&tmp).description().is_empty());
    }

    #[test]
    fn tool_schema_has_required_fields() {
        let tmp = TempDir::new().unwrap();
        let schema = make_tool(&tmp).parameters_schema();
        assert!(schema["properties"]["owner"].is_object());
        assert!(schema["properties"]["repo"].is_object());
        assert!(schema["properties"]["branch"].is_object());
        assert!(schema["properties"]["workdir"].is_object());
        let required = schema["required"].as_array().unwrap();
        let req_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
        assert!(req_strs.contains(&"owner"));
        assert!(req_strs.contains(&"repo"));
    }

    #[tokio::test]
    async fn returns_error_without_sandbox() {
        let tmp = TempDir::new().unwrap();
        let tool = make_tool(&tmp);
        store::init_db(&PathBuf::from(&tool.config.db_path)).unwrap();
        let result = tool
            .execute(json!({"owner": "acme", "repo": "test-repo"}))
            .await
            .unwrap();
        assert!(!result.success);
        // No sandbox — require_id fails first
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn returns_error_without_token() {
        let tmp = TempDir::new().unwrap();
        let tool = make_tool(&tmp);
        // Init DB (empty — no token)
        store::init_db(&PathBuf::from(&tool.config.db_path)).unwrap();
        // Pretend sandbox is active by setting sandbox id
        tool.client.set_id("fake-sandbox-id".to_string());
        let result = tool
            .execute(json!({"owner": "acme", "repo": "test-repo"}))
            .await
            .unwrap();
        assert!(!result.success);
        let err = result.error.as_deref().unwrap_or("");
        assert!(err.contains("not connected") || err.contains("github_connect"));
    }

    #[test]
    fn should_skip_binary_extensions() {
        assert!(should_skip("public/favicon.ico"));
        assert!(should_skip("assets/logo.png"));
        assert!(should_skip("fonts/Inter.woff2"));
        assert!(should_skip("dist/app.zip"));
    }

    #[test]
    fn should_skip_ignored_dirs() {
        assert!(should_skip("node_modules/react/index.js"));
        assert!(should_skip(".next/server/chunks/page.js"));
        assert!(should_skip("target/debug/app"));
        assert!(should_skip("dist/main.js"));
        assert!(should_skip(".git/config"));
    }

    #[test]
    fn should_not_skip_source_files() {
        assert!(!should_skip("src/main.rs"));
        assert!(!should_skip("src/app/page.tsx"));
        assert!(!should_skip("README.md"));
        assert!(!should_skip("package.json"));
        assert!(!should_skip("Cargo.toml"));
    }
}
