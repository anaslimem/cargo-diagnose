# Cargo Diagnose

`cargo-diagnose` is a robust command-line dependency health analyzer for Rust projects. It parses your `Cargo.toml` and cross-references your direct dependencies against three major threat intelligence sources:
- **OSV.dev** (Vulnerabilities and Security Advisories)
- **Crates.io** (Deprecations and Outdated Versions)
- **GitHub API** (Repository Maintenance Health and Archived state)

## Installation

You can install it directly from crates.io using Cargo:

```bash
cargo install cargo-diagnose
```
## Usage
Simply navigate to any Rust project directory (with a valid `Cargo.toml`) and run:

```bash
cargo-diagnose analyze
```

This will output a clean, human-readable terminal report grading your dependencies out of 100%:

```text
Dependency Health Check Report
==============================

Overall Health: 88%
Good Crates: 7/8
Problematic Crates: 1

Details:
---------------------------------------------------
Crate Name   : rustls
Repo         : github.com/rustls/rustls
Issue        : Outdated version (current: 0.23.37, latest: 0.24.0-dev.0)
Risk Type    : Version Risk
...
---------------------------------------------------
Missing / Vulnerable Crates: 12%
Good / Healthy Crates: 88%
```

### JSON Mode
If you want to plug this into a CI/CD pipeline or external dashboard, you can output pure JSON data instead:

```bash
cargo-diagnose analyze --json
```

### CI Threshold (Fail Under)
You can configure `cargo-diagnose` to exit with a non-zero system failure code if the project health dips below a certain percentage. This is incredibly useful for blocking Pull Requests that introduce heavily unmaintained or vulnerable crates:

```bash
cargo-diagnose analyze --fail-under 90
```
If the overall score is less than `90%`, it will fail the build step.

## How the Scoring Works
A direct dependency is considered "Healthy" natively unless it meets one of these severe conditions:
1. **Security Risk:** It has a reported CVE vulnerability on OSV.dev.
2. **Maintenance Risk:** The repository is officially "Archived" on GitHub, or it has zero stars and an alarming amount of open issues.
3. **Severe Issue Count:** It has more than 10 individual issues explicitly flagged by `cargo-doctor`.

If an update is available (Version Risk), it will be flagged in the report for your context, but it **will not** negatively alter your health percentage, to avoid unnecessary noise for fast-moving ecosystems.

## License
MIT