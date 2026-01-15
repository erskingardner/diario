use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::parser;
use crate::types::HomeworkEntry;

/// Process all export files and merge with existing data
pub fn process_all_exports(output_dir: &Path) -> Result<Vec<HomeworkEntry>> {
    let json_path = output_dir.join("homework.json");

    // Load existing entries
    let existing_entries = load_existing_entries(&json_path).unwrap_or_default();
    let existing_count = existing_entries.len();

    // Find and process all export files
    let files = find_all_exports()?;

    if files.is_empty() {
        if existing_entries.is_empty() {
            anyhow::bail!("No export files found in data/ and no existing data.");
        }
        println!("No export files found, using existing data.");
        return Ok(existing_entries);
    }

    let mut new_entries: Vec<HomeworkEntry> = Vec::new();
    for file in &files {
        println!("Processing: {}", file.display());
        match parser::parse_excel_xml(file) {
            Ok(entries) => {
                println!("  Found {} entries", entries.len());
                new_entries.extend(entries);
            }
            Err(e) => {
                eprintln!("  Warning: Failed to parse {}: {}", file.display(), e);
            }
        }
    }

    // Merge and deduplicate
    let all_entries = merge_and_deduplicate(existing_entries, new_entries);
    let new_count = all_entries.len().saturating_sub(existing_count);

    println!("Total entries: {} ({} new)", all_entries.len(), new_count);

    // Save updated JSON
    save_json(&all_entries, &json_path)?;
    println!("Data saved: {}", json_path.display());

    Ok(all_entries)
}

/// Load existing entries from JSON file
fn load_existing_entries(path: &PathBuf) -> Result<Vec<HomeworkEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path).context("Failed to read existing JSON")?;
    let entries: Vec<HomeworkEntry> =
        serde_json::from_str(&content).context("Failed to parse existing JSON")?;

    println!("Loaded {} existing entries", entries.len());
    Ok(entries)
}

/// Find all export files in data/ directory
fn find_all_exports() -> Result<Vec<PathBuf>> {
    let data_dir = PathBuf::from("data");

    if !data_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files: Vec<_> = std::fs::read_dir(&data_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("export_") && n.contains(".xls"))
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();

    files.sort();
    Ok(files)
}

/// Merge new entries with existing, removing duplicates
fn merge_and_deduplicate(
    existing: Vec<HomeworkEntry>,
    new: Vec<HomeworkEntry>,
) -> Vec<HomeworkEntry> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result: Vec<HomeworkEntry> = Vec::new();

    // Add existing entries first
    for entry in existing {
        let key = entry.dedup_key();
        if seen.insert(key) {
            result.push(entry);
        }
    }

    // Add new entries if not duplicates
    for entry in new {
        let key = entry.dedup_key();
        if seen.insert(key) {
            result.push(entry);
        }
    }

    // Sort by date
    result.sort_by(|a, b| a.date.cmp(&b.date));

    result
}

fn save_json(entries: &[HomeworkEntry], path: &PathBuf) -> Result<()> {
    let json = serde_json::to_string_pretty(entries)?;
    std::fs::write(path, json)?;
    Ok(())
}
