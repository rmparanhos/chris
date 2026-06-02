//! Notificações de Pull Request.
//!
//! Faz polling na API do GitHub procurando PRs que pedem a SUA revisão, avisa
//! quando aparecem novos e permite aprovar. A lógica de interpretar a resposta
//! e detectar "o que é novo" é pura (e testada); só `fetch`/`approve` tocam a
//! rede.

use std::collections::HashSet;

use serde::Deserialize;

const API: &str = "https://api.github.com";
const UA: &str = "chris-companion";

/// Um Pull Request esperando atenção.
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

/// Erros da integração.
#[derive(Debug)]
pub enum GhError {
    Http(String),
    Parse(String),
    NoToken,
}

// --- formato cru da resposta de /search/issues ---
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

/// Interpreta o JSON de `/search/issues` em uma lista de PRs. (Puro/testável.)
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

/// Extrai (owner, repo) de uma URL tipo `https://github.com/owner/repo/pull/5`.
fn owner_repo_from_url(url: &str) -> (String, String) {
    // pega o trecho depois de "github.com/"
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

/// Dado o que já foi visto, retorna só os PRs novos. (Puro/testável.)
pub fn only_new(seen: &HashSet<u64>, current: &[PrItem]) -> Vec<PrItem> {
    current
        .iter()
        .filter(|p| !seen.contains(&p.id))
        .cloned()
        .collect()
}

/// Descobre o token: env `GITHUB_TOKEN`/`GH_TOKEN`, ou `gh auth token`.
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
    // fallback: usa o login do GitHub CLI, se existir
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

/// Busca PRs que pedem a sua revisão (precisa de rede + token).
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

/// Aprova um PR (submete um review APPROVE). Precisa de rede + token.
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

/// Codificação mínima de querystring (só o que a query usa).
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
        assert_eq!(only_new(&seen, &prs).len(), 1); // novo
        seen.insert(99001);
        assert_eq!(only_new(&seen, &prs).len(), 0); // já visto
    }

    #[test]
    fn urlencode_spaces_and_at() {
        assert_eq!(urlencode("is:pr @me"), "is%3Apr%20%40me");
    }
}
