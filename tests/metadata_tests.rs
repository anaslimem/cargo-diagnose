use cargo_diagnose::metadata::parse_dependencies;
use serde_json::json;

#[test]
fn test_parse_dependencies() {
    let metadata_json = json!({
        "packages": [
            {
                "name": "direct-dep",
                "version": "1.0.0",
                "id": "direct-dep 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
                "dependencies": [],
                "targets": [],
                "features": {},
                "manifest_path": "/path/to/direct/Cargo.toml"
            },
            {
                "name": "transitive-dep",
                "version": "0.5.0",
                "id": "transitive-dep 0.5.0 (registry+https://github.com/rust-lang/crates.io-index)",
                "dependencies": [],
                "targets": [],
                "features": {},
                "manifest_path": "/path/to/transitive/Cargo.toml"
            },
            {
                "name": "my-pkg",
                "version": "0.1.0",
                "id": "my-pkg 0.1.0 (path+file:///path/to/my-pkg)",
                "dependencies": [],
                "targets": [],
                "features": {},
                "manifest_path": "/path/to/my-pkg/Cargo.toml"
            }
        ],
        "workspace_members": [
            "my-pkg 0.1.0 (path+file:///path/to/my-pkg)"
        ],
        "resolve": {
            "nodes": [
                {
                    "id": "my-pkg 0.1.0 (path+file:///path/to/my-pkg)",
                    "dependencies": [
                        "direct-dep 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    ]
                },
                {
                    "id": "direct-dep 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
                    "dependencies": [
                        "transitive-dep 0.5.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    ]
                }
            ]
        },
        "target_directory": "/path/to/target",
        "version": 1,
        "workspace_root": "/path/to/root"
    });

    let metadata: cargo_metadata::Metadata = serde_json::from_value(metadata_json).unwrap();
    let deps = parse_dependencies(&metadata);

    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "direct-dep");
    assert_eq!(deps[0].version, "1.0.0");
}
