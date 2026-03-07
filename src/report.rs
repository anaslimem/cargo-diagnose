use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskType {
    OK,
    SecurityRisk,
    MaintenanceRisk,
    VersionRisk,
}

impl fmt::Display for RiskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskType::OK => write!(f, "OK"),
            RiskType::SecurityRisk => write!(f, "Security Risk"),
            RiskType::MaintenanceRisk => write!(f, "Maintenance Risk"),
            RiskType::VersionRisk => write!(f, "Version Risk"),
        }
    }
}

pub struct CrateReport {
    pub name: String,
    pub repo: Option<String>,
    pub issues: Vec<String>,
    pub risk_type: RiskType,
    pub score: i32,
    highest_severity: i32, // Used internally to track the worst issue and its true risk_type
}

impl CrateReport {
    pub fn new(name: String, repo: Option<String>) -> Self {
        Self {
            name,
            repo,
            issues: Vec::new(),
            risk_type: RiskType::OK,
            score: 100, // Every crate starts with 100 points
            highest_severity: 0,
        }
    }

    pub fn add_issue(&mut self, issue: String, new_risk: RiskType, penalty: i32, severity: i32) {
        self.issues.push(issue);

        // Deduct points, but don't let a crate go below 0
        self.score = (self.score - penalty).max(0);

        // Only update the primary risk_type if this issue is more severe than previous ones
        if severity > self.highest_severity {
            self.highest_severity = severity;
            self.risk_type = new_risk;
        }
    }

    pub fn is_healthy(&self) -> bool {
        // We now consider a crate unhealthy if it lost any points
        // (meaning a version bump alone keeps it 100% healthy)
        self.score == 100
    }
}
