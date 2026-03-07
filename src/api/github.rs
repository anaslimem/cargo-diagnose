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
    let (owner, repo) = match parse_github_url(repo_url) {
        Some(res) => res,
        None => return Ok(None),
    };

    // Attempt to fetch repo info with retries
    crate::api::retry(
        || {
            let owner = owner.clone();
            let repo = repo.clone();
            async move {
                match octocrab.repos(owner, repo).get().await {
                    Ok(repo_stats) => Ok(Some(GithubRepoStats {
                        stars: repo_stats.stargazers_count.unwrap_or(0),
                        open_issues: repo_stats.open_issues_count.unwrap_or(0),
                        is_archived: repo_stats.archived.unwrap_or(false),
                    })),
                    Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
                }
            }
        },
        3,
    )
    .await
}

pub fn parse_github_url(url: &str) -> Option<(String, String)> {
    let parsed_url = Url::parse(url).ok()?;

    if parsed_url.host_str() != Some("github.com") {
        return None;
    }

    let segments: Vec<_> = parsed_url.path_segments()?.collect();

    if segments.len() < 2 {
        return None;
    }

    let owner = segments[0].to_string();
    let repo = segments[1].trim_end_matches(".git").to_string();

    if owner.is_empty() || repo.is_empty() {
        return None;
    }

    Some((owner, repo))
}
