use crate::filesystem;
use crate::finder::structures::{Opts as FinderOpts, SuggestionType};

use crate::common::git;
use crate::prelude::*;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct RepoMetadata {
    repo: String,
    description: String,
    tags: Vec<String>,
    stars: u32,
    last_updated: String,
    // Optionally include more fields such as category
    category: Option<String>,
}

pub fn main() -> Result<String> {
    let finder = CONFIG.finder();

    // Create temporary directory for cloning featured repos
    let repo_pathbuf = {
        let mut p = filesystem::tmp_pathbuf()?;
        p.push("featured");
        p
    };

    let repo_path_str = repo_pathbuf.to_str().ok_or_else(|| anyhow!("Invalid path"))?;

    let _ = filesystem::remove_dir(&repo_pathbuf);
    filesystem::create_dir(&repo_pathbuf)?;

    let (repo_url, _, _) = git::meta("grenkoca/cheats");
    git::shallow_clone(repo_url.as_str(), repo_path_str)
        .with_context(|| format!("Failed to clone `{repo_url}`"))?;

    // Expect a JSON file with structured metadata instead of a plain text file.
    let feature_repos_file = {
        let mut p = repo_pathbuf.clone();
        p.push("featured_repos.json");
        p
    };

    let json_content = fs::read_to_string(&feature_repos_file)
        .context("Unable to fetch featured repositories JSON file")?;

    let repos: Vec<RepoMetadata> = serde_json::from_str(&json_content)
        .context("Failed to parse featured repositories JSON")?;

    // Build a table string with headers. Each row will be tab-delimited.
    // Headers: Repo, Description, Tags, Stars, Last Updated
    let mut table = String::new();
    table.push_str("Repo\tDescription\tTags\tStars\tLast Updated\n");
    for repo in repos {
        let tags = repo.tags.join(", ");
        table.push_str(&format!("{}\t{}\t{}\t{}\t{}\n",
            repo.repo,
            repo.description,
            tags,
            repo.stars,
            repo.last_updated
        ));
    }

    // Finder options: set header lines and delimiter for sorting/filtering.
    // Additionally, you can pass fzf overrides to support sorting (e.g., toggling sort order).
    let opts = FinderOpts {
        header_lines: 1,
        delimiter: Some("\t".to_string()),
        column: Some(1), // default sort/display column (adjust as needed)
        suggestion_type: SuggestionType::SingleSelection,
        overrides: Some("--bind 'ctrl-s:toggle-sort'".to_string()),
        ..Default::default()
    };

    let (selected, _) = finder
        .call(opts, |stdin| {
            stdin
                .write_all(table.as_bytes())
                .context("Unable to prompt featured repositories")?;
            Ok(())
        })
        .context("Failed to get repo URL from finder")?;

    filesystem::remove_dir(&repo_pathbuf)?;

    // Extract the repo field (the first column) from the selected line.
    let repo_field = selected.split('\t').next().unwrap_or("").to_string();
    Ok(repo_field)
}

