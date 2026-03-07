use cargo_diagnose::report::{CrateReport, RiskType};

#[test]
fn test_new_report() {
    let report = CrateReport::new("test-crate".to_string(), Some("repo".to_string()));
    assert_eq!(report.name, "test-crate");
    assert_eq!(report.score, 100);
    assert_eq!(report.risk_type, RiskType::OK);
    assert!(report.is_healthy());
}

#[test]
fn test_add_issue_deducts_score() {
    let mut report = CrateReport::new("test-crate".to_string(), None);
    report.add_issue(
        "vulnerability".to_string(),
        RiskType::SecurityRisk,
        100,
        100,
    );
    assert_eq!(report.score, 0);
    assert!(!report.is_healthy());
    assert_eq!(report.risk_type, RiskType::SecurityRisk);
}

#[test]
fn test_score_floor_zero() {
    let mut report = CrateReport::new("test-crate".to_string(), None);
    report.add_issue("issue1".to_string(), RiskType::SecurityRisk, 60, 10);
    report.add_issue("issue2".to_string(), RiskType::MaintenanceRisk, 60, 5);
    assert_eq!(report.score, 0);
}

#[test]
fn test_highest_severity_wins() {
    let mut report = CrateReport::new("test-crate".to_string(), None);

    // Low severity version risk
    report.add_issue("old version".to_string(), RiskType::VersionRisk, 0, 10);
    assert_eq!(report.risk_type, RiskType::VersionRisk);

    // High severity security risk
    report.add_issue("CVE".to_string(), RiskType::SecurityRisk, 100, 100);
    assert_eq!(report.risk_type, RiskType::SecurityRisk);

    // Lower severity maintenance risk shouldn't override security risk
    report.add_issue("archived".to_string(), RiskType::MaintenanceRisk, 20, 50);
    assert_eq!(report.risk_type, RiskType::SecurityRisk);
}
