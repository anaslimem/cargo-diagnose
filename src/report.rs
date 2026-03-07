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
        self.issues.is_empty()
    }
}
