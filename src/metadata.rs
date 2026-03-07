use cargo_metadata::MetadataCommand;

#[derive(Clone)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
}

pub fn get_project_dependencies() -> Result<Vec<DependencyInfo>, Box<dyn std::error::Error>> {
    let metadata = MetadataCommand::new().exec()?;
    Ok(parse_dependencies(&metadata))
}

pub fn parse_dependencies(metadata: &cargo_metadata::Metadata) -> Vec<DependencyInfo> {
    let mut dependencies = Vec::new();
    let workspace_members = &metadata.workspace_members;
    let mut direct_dependency_ids = std::collections::HashSet::new();

    if let Some(resolve) = &metadata.resolve {
        for node in &resolve.nodes {
            if workspace_members.contains(&node.id) {
                for dep in &node.dependencies {
                    direct_dependency_ids.insert(dep);
                }
            }
        }
    }

    for package in &metadata.packages {
        if direct_dependency_ids.contains(&package.id) {
            dependencies.push(DependencyInfo {
                name: package.name.to_string(),
                version: package.version.to_string(),
            });
        }
    }

    dependencies
}
