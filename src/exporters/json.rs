use anyhow::Result;

use crate::models::RepoSnapshot;

pub fn export(snapshot: &RepoSnapshot) -> Result<String> {
    Ok(serde_json::to_string_pretty(snapshot)?)
}
