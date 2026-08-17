use anyhow::Result;

use crate::models::RepoSnapshot;

pub fn export_file_churn(snapshot: &RepoSnapshot) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record([
        "path",
        "commit_count",
        "additions",
        "deletions",
        "churn",
        "contributor_count",
    ])?;

    for file in &snapshot.file_churn {
        let row = vec![
            file.path.clone(),
            file.commit_count.to_string(),
            file.additions.to_string(),
            file.deletions.to_string(),
            file.total_churn().to_string(),
            file.contributor_count.to_string(),
        ];
        writer.write_record(&row)?;
    }

    let bytes = writer.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

pub fn export_contributors(snapshot: &RepoSnapshot) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record([
        "name",
        "commit_count",
        "lines_added",
        "lines_deleted",
        "files_touched",
    ])?;

    for contributor in &snapshot.contributors {
        let row = vec![
            contributor.name.clone(),
            contributor.commit_count.to_string(),
            contributor.lines_added.to_string(),
            contributor.lines_deleted.to_string(),
            contributor.files_touched.to_string(),
        ];
        writer.write_record(&row)?;
    }

    let bytes = writer.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}
