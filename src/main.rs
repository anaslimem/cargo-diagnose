use clap::{Parser, Subcommand};

mod api;
mod metadata;
mod report;

#[derive(Parser, Debug)]
#[command(name = "cargo-health")]
#[command(about = "Cargo Dependency Health Analyzer CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Runs a full dependency audit
    Analyze {
        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Fail if health score is below threshold
        #[arg(long)]
        fail_under: Option<u8>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Both octocrab and reqwest use rustls under the hood. Since multiple crates bring in
    // aws-lc-rs and ring, we need to explicitly initialize one of the crypto providers.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { json, fail_under } => {
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
            
            for dep in &dependencies {
                let mut report = report::CrateReport::new(dep.name.clone(), None);

                // 1. Query OSV for vulnerabilities
                match api::osv::check_vulnerabilities(&client, &dep.name, &dep.version).await {
                    Ok(osv_res) => {
                        if let Some(vulns) = osv_res.vulns {
                            for v in vulns {
                                let summary = v.summary.unwrap_or_else(|| "Unknown".to_string());
                                report.add_issue(format!("Security - {}", v.id), "Security Risk");
                            }
                        }
                    }
                    Err(_) => {}
                }

                // 2. Query Crates.io for latest version / metadata
                match api::crates_io::get_crate_info(&client, &dep.name).await {
                    Ok(crates_res) => {
                        if crates_res.crate_data.max_version != dep.version {
                            report.add_issue(
                                format!("Outdated version (current: {}, latest: {})", dep.version, crates_res.crate_data.max_version),
                                "Version Risk"
                            );
                        }
                        
                        if let Some(repo_url) = crates_res.crate_data.repository {
                            // Trim https:// and www. for clean printing
                            let clean_repo = repo_url
                                .replace("https://", "")
                                .replace("http://", "")
                                .replace("www.", "");
                            report.repo = Some(clean_repo);
                            
                            // 3. Query GitHub if it's a github repo
                            if let Ok(Some(stats)) = api::github::get_repo_stats(&repo_url).await {
                                if stats.is_archived {
                                    report.add_issue("Repository is Archived".to_string(), "Maintenance Risk");
                                } else if stats.stars == 0 && stats.open_issues > 100 {
                                    // Soft heuristic warning
                                    report.add_issue("High open issues vs stars".to_string(), "Maintenance Risk");
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }

                reports.push(report);
            }
            
            let total = reports.len();
            let healthy: usize = reports.iter().filter(|r| r.is_healthy()).count();
            let problematic = total - healthy;
            
            let overall_health = if total == 0 { 100.0 } else { (healthy as f64 / total as f64) * 100.0 };
            
            if !json {
                println!("\nDependency Health Check Report");
                println!("==============================");
                println!("");
                println!("Overall Health: {:.0}%", overall_health);
                println!("Good Crates: {}/{}", healthy, total);
                println!("Problematic Crates: {}", problematic);
                println!("");
                println!("Details:");
                
                for report in &reports {
                    println!("---------------------------------------------------");
                    println!("{:<13}: {}", "Crate Name", report.name);
                    println!("{:<13}: {}", "Repo", report.repo.as_deref().unwrap_or("Unknown"));
                    
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
                println!("Missing / Vulnerable Crates: {:.0}%", 100.0 - overall_health);
                println!("Good / Healthy Crates: {:.0}%", overall_health);
            } else {
                // If json mode is enabled
                let json_output = serde_json::json!({
                    "overall_health": overall_health,
                    "good_crates": healthy,
                    "problematic_crates": problematic,
                    "total": total
                });
                println!("{}", json_output.to_string());
            }

            if let Some(threshold) = fail_under {
                if (overall_health as u8) < threshold {
                    if !json {
                        eprintln!("\nHealth score {:.0}% is below threshold of {}%.", overall_health, threshold);
                    }
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}
