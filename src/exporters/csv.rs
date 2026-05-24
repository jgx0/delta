use anyhow::Result;

use crate::models::RepoSnapshot;

pub fn export_file_churn(snapshot: &RepoSnapshot) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(["path", "commit_count", "additions", "deletions", "churn"])?;

    for file in &snapshot.file_churn {
        writer.write_record([
            file.path.as_str(),
            &file.commit_count.to_string(),
            &file.additions.to_string(),
            &file.deletions.to_string(),
            &file.total_churn().to_string(),
        ])?;
    }

    let bytes = writer.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}
