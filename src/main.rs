use clap::{Parser, Subcommand};

mod api;
mod metadata;
mod report;

#[derive(Parser, Debug)]
#[command(name = "cargo-diagnose")]
#[command(about = "Cargo Dependency Health Analyzer CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Support for `cargo diagnose` alias.
    /// This is the primary entry point when used as a cargo plugin.
    #[command(name = "diagnose", hide = true)]
    Diagnose(AnalyzeArgs),
    /// Analyze your project's dependencies for health and security risks.
    Analyze(AnalyzeArgs),
}

#[derive(clap::Args, Debug, Clone)]
struct AnalyzeArgs {
    /// Output the health report result as a JSON object
    #[arg(long)]
    json: bool,

    /// Fail the command (exit code 1) if the overall health score is below this threshold (0-100)
    #[arg(long)]
    fail_under: Option<u8>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Both octocrab and reqwest use rustls under the hood. Since multiple crates bring in
    // aws-lc-rs and ring, we need to explicitly initialize one of the crypto providers.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze(args) | Commands::Diagnose(args) => {
            let json = args.json;
            let fail_under = args.fail_under;
            if !json {
                println!("Scanning project...");
            }

            let dependencies = match metadata::get_project_dependencies() {
                Ok(deps) => deps,
                Err(e) => {
                    eprintln!("Failed to parse Cargo metadata: {}", e);
                    std::process::exit(1);
                }
            };

            if !json {
                println!("Analyzing {} dependencies...", dependencies.len());
            }

            let client = reqwest::Client::new();
            let mut reports: Vec<report::CrateReport> = Vec::new();

            // Initialize octocrab once and share it
            let mut octocrab_builder = octocrab::Octocrab::builder();
            if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                octocrab_builder = octocrab_builder.personal_token(token);
            }
            let octocrab = std::sync::Arc::new(octocrab_builder.build()?);

            let mut join_set = tokio::task::JoinSet::new();

            for dep in dependencies {
                let client = client.clone();
                let octocrab = std::sync::Arc::clone(&octocrab);

                join_set.spawn(async move {
                    let mut report = report::CrateReport::new(dep.name.clone(), None);

                    // 1. Query OSV for vulnerabilities
                    match api::osv::check_vulnerabilities(&client, &dep.name, &dep.version).await {
                        Ok(osv_res) => {
                            if let Some(vulns) = osv_res.vulns {
                                for v in vulns {
                                    report.add_issue(
                                        format!("Security - {}", v.id),
                                        report::RiskType::SecurityRisk,
                                        100,
                                        100,
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to check vulnerabilities for {}: {}",
                                dep.name, e
                            );
                        }
                    }

                    // 2. Query Crates.io for latest version / metadata
                    match api::crates_io::get_crate_info(&client, &dep.name).await {
                        Ok(crates_res) => {
                            if crates_res.crate_data.max_version != dep.version {
                                report.add_issue(
                                    format!(
                                        "Outdated version (current: {}, latest: {})",
                                        dep.version, crates_res.crate_data.max_version
                                    ),
                                    report::RiskType::VersionRisk,
                                    0,
                                    10,
                                );
                            }

                            if let Some(repo_url) = crates_res.crate_data.repository {
                                let clean_repo = repo_url
                                    .replace("https://", "")
                                    .replace("http://", "")
                                    .replace("www.", "");
                                report.repo = Some(clean_repo);

                                // 3. Query GitHub if it's a github repo
                                match api::github::get_repo_stats(&octocrab, &repo_url).await {
                                    Ok(Some(stats)) => {
                                        if stats.is_archived {
                                            report.add_issue(
                                                "Repository is Archived".to_string(),
                                                report::RiskType::MaintenanceRisk,
                                                100,
                                                50,
                                            );
                                        } else if stats.stars == 0 && stats.open_issues > 100 {
                                            report.add_issue(
                                                "High open issues vs stars".to_string(),
                                                report::RiskType::MaintenanceRisk,
                                                20,
                                                30,
                                            );
                                        }
                                    }
                                    Ok(None) => {} // Not a GitHub repo
                                    Err(e) => {
                                        eprintln!(
                                            "Warning: Failed to get GitHub stats for {}: {}",
                                            dep.name, e
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to get Crates.io info for {}: {}",
                                dep.name, e
                            );
                        }
                    }
                    report
                });
            }

            while let Some(res) = join_set.join_next().await {
                if let Ok(report) = res {
                    reports.push(report);
                }
            }

            // Optional: Sort reports alphabetically by crate name for consistent output
            reports.sort_by(|a, b| a.name.cmp(&b.name));

            let total = reports.len();
            let healthy: usize = reports.iter().filter(|r| r.is_healthy()).count();
            let problematic = total - healthy;

            let overall_health = if total == 0 {
                100.0
            } else {
                let total_score: i32 = reports.iter().map(|r| r.score).sum();
                (total_score as f64) / (total as f64)
            };

            if !json {
                println!("\nDependency Health Check Report");
                println!("==============================");
                println!();
                println!("Overall Health: {:.0}%", overall_health);
                println!("Good Crates: {}/{}", healthy, total);
                println!("Problematic Crates: {}", problematic);
                println!();
                println!("Details:");

                for report in &reports {
                    println!("---------------------------------------------------");
                    println!("{:<13}: {}", "Crate Name", report.name);
                    println!("{:<13}: {}", "Score", report.score);
                    println!(
                        "{:<13}: {}",
                        "Repo",
                        report.repo.as_deref().unwrap_or("Unknown")
                    );

                    if report.issues.is_empty() {
                        println!("{:<13}: None", "Issue");
                        println!("{:<13}: OK", "Risk Type");
                    } else {
                        // Print the first issue for brevity as requested
                        println!("{:<13}: {}", "Issue", report.issues[0]);
                        println!("{:<13}: {}", "Risk Type", report.risk_type);
                    }
                }
                println!("---------------------------------------------------");
                println!(
                    "Missing / Vulnerable Crates: {:.0}%",
                    100.0 - overall_health
                );
                println!("Good / Healthy Crates: {:.0}%", overall_health);
            } else {
                // If json mode is enabled
                let json_output = serde_json::json!({
                    "overall_health": overall_health,
                    "good_crates": healthy,
                    "problematic_crates": problematic,
                    "total": total
                });
                println!("{}", json_output);
            }

            if let Some(threshold) = fail_under
                && (overall_health as u8) < threshold
            {
                if !json {
                    eprintln!(
                        "\nHealth score {:.0}% is below threshold of {}%.",
                        overall_health, threshold
                    );
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
