pub struct CrateReport {
    pub name: String,
    pub repo: Option<String>,
    pub issues: Vec<String>,
    pub risk_type: String,
}

impl CrateReport {
    pub fn new(name: String, repo: Option<String>) -> Self {
        Self {
            name,
            repo,
            issues: Vec::new(),
            risk_type: "OK".to_string(),
        }
    }
    
    pub fn add_issue(&mut self, issue: String, new_risk: &str) {
        self.issues.push(issue);
        self.risk_type = new_risk.to_string();
    }
    
    pub fn is_healthy(&self) -> bool {
        // A crate is only considered problematic if it has:
        // - A security risk
        // - A maintenance risk (e.g. archived)
        // - More than 10 total issues (if we tracked individual GitHub issues)
        // A simple version bump shouldn't immediately mark the crate as "Problematic".
        
        if self.risk_type == "Security Risk" || self.risk_type == "Maintenance Risk" || self.issues.len() > 10 {
            return false;
        }

        true
    }
}
