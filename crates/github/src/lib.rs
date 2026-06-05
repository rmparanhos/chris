//! Pull Request notifications.
//!
//! Polls the GitHub API looking for PRs that request YOUR review, notifies
//! when new ones appear, and lets you approve them. The logic that parses the
//! response and detects "what is new" is pure (and tested); only
//! `fetch`/`approve` touch the network.

use std::collections::HashSet;

use serde::Deserialize;

const API: &str = "https://api.github.com";
const UA: &str = "chris-companion";

/// A Pull Request awaiting attention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrItem {
    pub id: u64,
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub author: String,
    pub url: String,
}

/// Integration errors.
#[derive(Debug)]
pub enum GhError {
    Http(String),
    Parse(String),
    NoToken,
}

// --- raw format of the /search/issues response ---
#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    items: Vec<SearchItem>,
}
#[derive(Deserialize)]
struct SearchItem {
    id: u64,
    number: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    user: User,
}
#[derive(Deserialize, Default)]
struct User {
    #[serde(default)]
    login: String,
}

/// Parses the JSON from `/search/issues` into a list of PRs. (Pure/testable.)
pub fn parse_search(json: &str) -> Result<Vec<PrItem>, GhError> {
    let resp: SearchResponse =
        serde_json::from_str(json).map_err(|e| GhError::Parse(e.to_string()))?;
    Ok(resp
        .items
        .into_iter()
        .map(|it| {
            let (owner, repo) = owner_repo_from_url(&it.html_url);
            PrItem {
                id: it.id,
                owner,
                repo,
                number: it.number,
                title: it.title,
                author: it.user.login,
                url: it.html_url,
            }
        })
        .collect())
}

/// Extracts (owner, repo) from a URL like `https://github.com/owner/repo/pull/5`.
fn owner_repo_from_url(url: &str) -> (String, String) {
    // grab the part after "github.com/"
    let tail = url
        .split("github.com/")
        .nth(1)
        .or_else(|| url.split("/repos/").nth(1))
        .unwrap_or("");
    let mut parts = tail.split('/');
    let owner = parts.next().unwrap_or("").to_string();
    let repo = parts.next().unwrap_or("").to_string();
    (owner, repo)
}

/// Given what has already been seen, returns only the new PRs. (Pure/testable.)
pub fn only_new(seen: &HashSet<u64>, current: &[PrItem]) -> Vec<PrItem> {
    current
        .iter()
        .filter(|p| !seen.contains(&p.id))
        .cloned()
        .collect()
}

/// Discovers the token: env `GITHUB_TOKEN`/`GH_TOKEN`, or `gh auth token`.
pub fn discover_token() -> Option<String> {
    if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    if let Ok(t) = std::env::var("GH_TOKEN") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    // fallback: use the GitHub CLI login, if it exists
    let out = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()?;
    if out.status.success() {
        let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    None
}

/// Fetches PRs that request your review (needs network + token).
pub fn fetch_review_requests(token: &str) -> Result<Vec<PrItem>, GhError> {
    let url = format!(
        "{API}/search/issues?q={}",
        urlencode("is:open is:pr review-requested:@me")
    );
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", UA)
        .call()
        .map_err(|e| GhError::Http(e.to_string()))?;
    let body = resp
        .into_string()
        .map_err(|e| GhError::Http(e.to_string()))?;
    parse_search(&body)
}

/// Approves a PR (submits an APPROVE review). Needs network + token.
pub fn approve_pr(token: &str, owner: &str, repo: &str, number: u64) -> Result<(), GhError> {
    let url = format!("{API}/repos/{owner}/{repo}/pulls/{number}/reviews");
    ureq::post(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", UA)
        .send_json(serde_json::json!({ "event": "APPROVE" }))
        .map_err(|e| GhError::Http(e.to_string()))?;
    Ok(())
}

/// Minimal querystring encoding (only what the query uses).
fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "total_count": 1,
        "items": [
            {
                "id": 99001,
                "number": 42,
                "title": "Corrige o parser",
                "html_url": "https://github.com/acme/widget/pull/42",
                "user": { "login": "alice" }
            }
        ]
    }"#;

    #[test]
    fn parse_extracts_pr() {
        let prs = parse_search(SAMPLE).unwrap();
        assert_eq!(prs.len(), 1);
        let p = &prs[0];
        assert_eq!(p.id, 99001);
        assert_eq!(p.owner, "acme");
        assert_eq!(p.repo, "widget");
        assert_eq!(p.number, 42);
        assert_eq!(p.title, "Corrige o parser");
        assert_eq!(p.author, "alice");
    }

    #[test]
    fn only_new_filters_seen() {
        let prs = parse_search(SAMPLE).unwrap();
        let mut seen = HashSet::new();
        assert_eq!(only_new(&seen, &prs).len(), 1); // new
        seen.insert(99001);
        assert_eq!(only_new(&seen, &prs).len(), 0); // already seen
    }

    #[test]
    fn urlencode_spaces_and_at() {
        assert_eq!(urlencode("is:pr @me"), "is%3Apr%20%40me");
    }
}
