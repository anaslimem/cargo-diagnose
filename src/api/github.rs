use octocrab::Octocrab;

#[derive(Debug)]
pub struct GithubRepoStats {
    pub stars: u32,
    pub open_issues: u32,
    pub is_archived: bool,
}

pub async fn get_repo_stats(
    octocrab: &Octocrab,
    repo_url: &str,
) -> Result<Option<GithubRepoStats>, Box<dyn std::error::Error + Send + Sync>> {
    // Basic heuristic to parse GitHub owner and repo from URLs
    // Example: "https://github.com/dtolnay/serde" -> owner: "dtolnay", repo: "serde"
    if !repo_url.contains("github.com") {
        return Ok(None);
    }

    let url = repo_url.trim_end_matches(".git");
    let parts: Vec<&str> = url.split("github.com/").collect();
    if parts.len() < 2 {
        return Ok(None);
    }

    let path_parts: Vec<&str> = parts[1].split('/').collect();
    if path_parts.len() < 2 {
        return Ok(None);
    }

    let owner = path_parts[0];
    let repo = path_parts[1];

    // Attempt to fetch repo
    match octocrab.repos(owner, repo).get().await {
        Ok(repo_stats) => Ok(Some(GithubRepoStats {
            stars: repo_stats.stargazers_count.unwrap_or(0),
            open_issues: repo_stats.open_issues_count.unwrap_or(0),
            is_archived: repo_stats.archived.unwrap_or(false),
        })),
        Err(e) => {
            // It could be a 404 if the repo was renamed or deleted.
            Err(Box::new(e))
        }
    }
}
