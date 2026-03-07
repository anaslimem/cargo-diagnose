# Cargo Diagnose

`cargo-diagnose` is a tool that checks the health of your Rust project's dependencies. It looks at your `Cargo.toml` file and checks your dependencies using:
- **OSV.dev** (for known security problems)
- **Crates.io** (for deprecated and old versions)
- **GitHub API** (to see if the repository is maintained or archived)

## Installation

You can install it directly from crates.io using Cargo:

```bash
cargo install cargo-diagnose
```

## Usage

Go to any Rust project directory (where your `Cargo.toml` is) and run:

```bash
cargo-diagnose analyze
```

This will print a report showing a score out of 100% for your dependencies:

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

### JSON Output

If you want to use this in scripts or other tools, you can get the output as JSON:

```bash
cargo-diagnose analyze --json
```

### Fail Test (CI)

You can make `cargo-diagnose` fail the command if the score is too low. This is useful for stopping pull requests that add unsafe or unmaintained crates:

```bash
cargo-diagnose analyze --fail-under 90
```
If the overall score is less than `90%`, the command will fail.

## How the Scoring Works

A dependency is healthy unless:
1. **Security Risk:** It has a known security problem on OSV.dev.
2. **Maintenance Risk:** The repository is archived on GitHub, or it has zero stars and many open issues.
3. **Many Issues:** It has more than 10 issues found by `cargo-doctor`.

If a newer version is available, it will be shown in the report. It will not lower your score, so you don't get warnings that are not important.

## License

MIT