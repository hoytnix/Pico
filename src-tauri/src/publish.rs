use crate::models::{PersistedAppSettings, PublishResult};
use std::path::Path;
use std::process::Command;

fn parse_github_owner_repo(remote_url: &str) -> Option<(String, String)> {
    let trimmed = remote_url.trim().trim_end_matches(".git");
    if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        let mut parts = rest.split('/');
        let owner = parts.next()?.to_string();
        let repo = parts.next()?.to_string();
        if !owner.is_empty() && !repo.is_empty() {
            return Some((owner, repo));
        }
    } else if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        let mut parts = rest.split('/');
        let owner = parts.next()?.to_string();
        let repo = parts.next()?.to_string();
        if !owner.is_empty() && !repo.is_empty() {
            return Some((owner, repo));
        }
    }
    None
}

pub async fn publish_to_github_pages(
    site_root: &Path,
    settings: &PersistedAppSettings,
) -> PublishResult {
    let out_dir = site_root.join("out");
    if !out_dir.is_dir() {
        return PublishResult {
            success: false,
            message: "Site has not been built yet — click Build first, or use Export.".to_string(),
            url: None,
        };
    }

    // 1. Check if `git` binary is available
    let git_version = Command::new("git").arg("--version").output();
    if git_version.is_err() || !git_version.unwrap().status.success() {
        return PublishResult {
            success: false,
            message: "Git is not installed on this computer — use Export to save your built site folder.".to_string(),
            url: None,
        };
    }

    // 2. Check if site_root is a git repository
    let is_repo = Command::new("git")
        .arg("-C")
        .arg(site_root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();
    if is_repo.is_err() || !is_repo.unwrap().status.success() {
        return PublishResult {
            success: false,
            message: "This site folder is not a git repository — use Export, or initialize a git repository with a GitHub remote.".to_string(),
            url: None,
        };
    }

    // 3. Check for a GitHub remote (`origin`)
    let remote_out = Command::new("git")
        .arg("-C")
        .arg(site_root)
        .args(["remote", "get-url", "origin"])
        .output();

    let remote_url = match remote_out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => String::new(),
    };

    if remote_url.is_empty() {
        return PublishResult {
            success: false,
            message: "No git remote found — use Export, or connect GitHub in Settings".to_string(),
            url: None,
        };
    }

    let (owner, repo) = match parse_github_owner_repo(&remote_url) {
        Some(pair) => pair,
        None => {
            return PublishResult {
                success: false,
                message: format!(
                    "Remote '{}' is not a GitHub repository — use Export, or set origin to a GitHub repository.",
                    remote_url
                ),
                url: None,
            }
        }
    };

    // 4. Commit and push `out/` to `gh-pages` branch using a temporary git repo inside `out/`
    let git_in_out = out_dir.join(".git");
    if git_in_out.exists() {
        let _ = std::fs::remove_dir_all(&git_in_out);
    }

    let init_ok = Command::new("git")
        .arg("-C")
        .arg(&out_dir)
        .args(["init", "-b", "gh-pages"])
        .output();
    if init_ok.is_err() || !init_ok.unwrap().status.success() {
        return PublishResult {
            success: false,
            message: "Could not initialize temporary gh-pages branch — use Export instead.".to_string(),
            url: None,
        };
    }

    let _ = Command::new("git")
        .arg("-C")
        .arg(&out_dir)
        .args(["config", "user.name", "Pico Publisher"])
        .output();
    let _ = Command::new("git")
        .arg("-C")
        .arg(&out_dir)
        .args(["config", "user.email", "pico@localhost"])
        .output();
    let _ = Command::new("git")
        .arg("-C")
        .arg(&out_dir)
        .args(["add", "-A"])
        .output();
    let _ = Command::new("git")
        .arg("-C")
        .arg(&out_dir)
        .args(["commit", "-m", "Publish site via Pico"])
        .output();

    let push_res = Command::new("git")
        .arg("-C")
        .arg(&out_dir)
        .args(["push", "--force", &remote_url, "HEAD:refs/heads/gh-pages"])
        .output();

    let _ = std::fs::remove_dir_all(&git_in_out);

    match push_res {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            return PublishResult {
                success: false,
                message: format!(
                    "Could not push to gh-pages branch ({}) — use Export, or check your GitHub credentials.",
                    stderr.lines().next().unwrap_or("authentication failed")
                ),
                url: None,
            };
        }
        Err(e) => {
            return PublishResult {
                success: false,
                message: format!("Failed to run git push ({}) — use Export.", e),
                url: None,
            }
        }
    }

    // 5. If GitHub token is configured in Settings, ensure GitHub Pages is enabled on `gh-pages` branch
    if !settings.github_token.trim().is_empty() {
        let api_url = format!("https://api.github.com/repos/{}/{}/pages", owner, repo);
        let client = reqwest::Client::new();
        let _ = client
            .post(&api_url)
            .bearer_auth(settings.github_token.trim())
            .header("User-Agent", "Pico-SSG")
            .header("Accept", "application/vnd.github+json")
            .json(&serde_json::json!({
                "source": {
                    "branch": "gh-pages",
                    "path": "/"
                }
            }))
            .send()
            .await;
    }

    let pages_url = format!("https://{}.github.io/{}/", owner, repo);
    PublishResult {
        success: true,
        message: format!("Published `out/` to `gh-pages` branch on {}/{}.", owner, repo),
        url: Some(pages_url),
    }
}
