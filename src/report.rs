pub struct CrateReport {
    pub name: String,
    pub repo: Option<String>,
    pub issues: Vec<String>,
    pub risk_type: String,
    pub score: i32,
    highest_severity: i32, // Used internally to track the worst issue and its true risk_type
}

impl CrateReport {
    pub fn new(name: String, repo: Option<String>) -> Self {
        Self {
            name,
            repo,
            issues: Vec::new(),
            risk_type: "OK".to_string(),
            score: 100, // Every crate starts with 100 points
            highest_severity: 0,
        }
    }

    pub fn add_issue(&mut self, issue: String, new_risk: &str, penalty: i32, severity: i32) {
        self.issues.push(issue);

        // Deduct points, but don't let a crate go below 0
        self.score = (self.score - penalty).max(0);

        // Only update the primary risk_type if this issue is more severe than previous ones
        if severity > self.highest_severity {
            self.highest_severity = severity;
            self.risk_type = new_risk.to_string();
        }
    }

    pub fn is_healthy(&self) -> bool {
        // We now consider a crate unhealthy if it lost any points
        // (meaning a version bump alone keeps it 100% healthy)
        self.score == 100
    }
}
