use octocrab::Octocrab;
use url::Url;

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
    // Parse the URL using the `url` crate for robustness
    let parsed_url = match Url::parse(repo_url) {
        Ok(url) => url,
        Err(_) => return Ok(None),
    };

    if parsed_url.host_str() != Some("github.com") {
        return Ok(None);
    }

    // Extract owner and repo from path (e.g., "/owner/repo")
    let segments: Vec<_> = parsed_url
        .path_segments()
        .map(|s| s.collect::<Vec<_>>())
        .unwrap_or_default();

    if segments.len() < 2 {
        return Ok(None);
    }

    let owner = segments[0];
    let repo = segments[1].trim_end_matches(".git");

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
