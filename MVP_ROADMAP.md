# Cargo Doctor (Health) - MVP Roadmap

This document outlines the step-by-step roadmap to build, test, and publish the Minimum Viable Product (MVP) for the `cargo-doctor` (or `cargo-health`) CLI tool.

## Phase 1: Project Initialization & Core Structure
**Goal:** Setup the repository and basic CLI skeleton.
1. **Initialize Project:** 
   - Run `cargo init` to create a new binary crate.
   - Configure basic package metadata (name, version, authors, description, keywords, categories) in `Cargo.toml`.
2. **Setup Dependencies:**
   - Install core crates: `clap` (CLI framing), `tokio` (async runtime), `reqwest` (HTTP requests), `serde`/`serde_json` (parsing).
3. **Build CLI Interface:**
   - Define the `cargo-health analyze` command using `clap`.
   - Add flags like `--json` and `--fail-under`.
   - Ensure the command outputs a basic "Scanning project..." placeholder to verify CLI wiring.

## Phase 2: Dependency Parsing
**Goal:** Extract the project dependency tree accurately using `cargo metadata`.
1. **Integrate `cargo_metadata`:** 
   - Use the `cargo_metadata` crate to locate the user's `Cargo.toml`/`Cargo.lock` in the current working directory.
2. **Extract Dependencies:**
   - Parse all direct and transitive dependencies.
   - Gather exact versions used via the lockfile.
3. **Display Parsed Data:**
   - Output a list of detected dependencies and their versions to ensure accurate reading before we start analyzing them.

## Phase 3: External Signals & Threat Intelligence
**Goal:** Fetch vulnerability, maintenance, and deprecation data for each dependency.
1. **OSV.dev Integration (Vulnerabilities):**
   - Query the OSV REST API (`api.osv.dev/v1/query`) matching the `crates.io` ecosystem and package version.
   - Parse responses to identify security severity and advisory IDs.
2. **Crates.io API Integration (Deprecations & Updates):**
   - Query `crates.io/api/v1/crates/{crate_name}`.
   - Check if the version is outdated by comparing with the latest release.
   - Check if the crate is marked as deprecated.
3. **GitHub API Integration (Repository Health):**
   - Resolve the repository URL from the crates.io metadata.
   - Query the GitHub REST API (using octocrab or raw reqwest) to fetch last commit date, issue count, stargazers, and archived status.
4. **Concurrency:**
   - Use `tokio::spawn` or `futures::future::join_all` to fetch signals for all dependencies concurrently without blocking.

## Phase 4: Scoring Engine & Output Generation
**Goal:** Grade the project dependencies and output actionable terminal reports.
1. **Scoring Logic:**
   - Implement the scoring formula (e.g., -50 for vulnerabilities, -40 for deprecation, -10 for outdated, starting at 100).
   - Cap scores between `0` and `100`.
   - Identify reasonings and warnings for deducted points.
2. **CLI Output Formatting:**
   - Print the scan results in a clear, human-readable terminal format.
   - Display a global project health score.
3. **JSON & CI Output Modes:**
   - Implement the `--json` output structure.
   - Implement the `--fail-under` functionality, exiting with a non-zero exit code (`std::process::exit(1)`) if the score is too low.

## Phase 5: Local Testing & Cross-Project Validation
**Goal:** Verify the tool works on different kinds of Rust projects before publishing.
1. **Self-Testing:** 
   - Run `cargo run -- analyze` within the `cargo-doctor` project itself to test against its own dependencies.
2. **Global Install Simulation:**
   - Run `cargo install --path .` locally to install the binary to `~/.cargo/bin/`.
   - Check out different Rust projects on your machine and run `cargo-health analyze` via terminal to verify it can successfully locate and parse foreign manifest files.

## Phase 6: Publishing to Crates.io
**Goal:** Package and release the tool to the global registry so others can install it via `cargo install cargo-health`.
1. **Prepare Release Data:**
   - Write a compelling `README.md` containing usage examples (gif/screenshots help!), installation instructions, and data source citations.
   - Choose a fitting open-source license (`MIT` or `Apache-2.0`).
   - Add `readme = "README.md"` and `license = "MIT"` into `Cargo.toml`.
2. **CI Pipeline Integration (Optional but recommended):**
   - Add basic GitHub Actions to run `cargo fmt`, `cargo clippy`, and `cargo test` on push.
3. **Final Verification:** 
   - Run `cargo publish --dry-run` to detect any missing fields or packaging errors.
4. **Publish:**
   - Run `cargo publish` to push `cargo-doctor` or `cargo-health` to `crates.io`.
   - You can now install it globally on any machine using `cargo install <crate-name>`.

## Phase 7: Post-Publish Roadmap
- Implement caching to prevent rate-limiting against crates.io / OSV / GitHub.
- Configurable rules (e.g., ignoring specific advisories or crates).
- Offline capabilities (downloading vulnerability DBs locally).
