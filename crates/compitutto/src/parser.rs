use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::HomeworkEntry;

/// Parse an Excel XML (SpreadsheetML) file and extract homework entries
pub fn parse_excel_xml(path: &Path) -> Result<Vec<HomeworkEntry>> {
    let content = fs::read_to_string(path).context("Failed to read file")?;

    // Check if it's XML format
    if !content.starts_with("<?xml") && !content.contains("<Workbook") {
        anyhow::bail!("File does not appear to be Excel XML format");
    }

    let rows = parse_spreadsheet_rows(&content)?;

    if rows.is_empty() {
        anyhow::bail!("No data rows found in file");
    }

    // First row is headers
    let headers = &rows[0];
    println!("Found columns: {:?}", headers);

    // Map column indices
    let col_indices = map_columns(headers);

    // Parse data rows into entries
    let mut entries = Vec::new();

    for row in rows.iter().skip(1) {
        if let Some(entry) = parse_row(row, &col_indices) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Parse SpreadsheetML XML into rows of cell values
fn parse_spreadsheet_rows(xml: &str) -> Result<Vec<Vec<String>>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut in_row = false;
    let mut in_cell = false;
    let mut in_data = false;
    let mut current_text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"Row" => {
                        in_row = true;
                        current_row = Vec::new();
                    }
                    b"Cell" => {
                        if in_row {
                            in_cell = true;
                        }
                    }
                    b"Data" => {
                        if in_cell {
                            in_data = true;
                            current_text.clear();
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"Row" => {
                        if in_row && !current_row.is_empty() {
                            rows.push(current_row.clone());
                        }
                        in_row = false;
                    }
                    b"Cell" => {
                        if in_cell && !in_data {
                            // Empty cell
                            current_row.push(String::new());
                        }
                        in_cell = false;
                    }
                    b"Data" => {
                        if in_data {
                            current_row.push(current_text.trim().to_string());
                            current_text.clear();
                        }
                        in_data = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if in_data {
                    if let Ok(text) = e.unescape() {
                        current_text.push_str(&text);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("XML parse error: {}", e),
            _ => {}
        }
    }

    Ok(rows)
}

/// Map header names to column indices
fn map_columns(headers: &[String]) -> HashMap<&'static str, usize> {
    let mut indices = HashMap::new();

    for (i, header) in headers.iter().enumerate() {
        let lower = header.to_lowercase();

        // Date column
        if lower.contains("data") || lower.contains("inizio") || lower.contains("date") {
            indices.entry("date").or_insert(i);
        }

        // Subject column
        if lower.contains("materia") || lower.contains("subject") || lower.contains("corso") {
            indices.entry("subject").or_insert(i);
        }

        // Task/description column
        if lower.contains("nota")
            || lower.contains("descrizione")
            || lower.contains("task")
            || lower.contains("compito")
        {
            indices.entry("task").or_insert(i);
        }

        // Type column (but not "tipo evento")
        if lower == "tipo" || (lower.contains("tipo") && !lower.contains("evento")) {
            indices.entry("type").or_insert(i);
        }
    }

    indices
}

/// Parse a single row into a HomeworkEntry
fn parse_row(row: &[String], col_indices: &HashMap<&'static str, usize>) -> Option<HomeworkEntry> {
    let get_col = |key: &str| -> String {
        col_indices
            .get(key)
            .and_then(|&i| row.get(i))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };

    let entry_type = get_col("type");
    let date = normalize_date(&get_col("date"));
    let subject = get_col("subject");
    let task = get_col("task");

    // Only include entries with meaningful data
    if task.is_empty() && subject.is_empty() {
        return None;
    }

    Some(HomeworkEntry::new(entry_type, date, subject, task))
}

/// Normalize date to YYYY-MM-DD format
fn normalize_date(date: &str) -> String {
    // If it contains a space (datetime), take just the date part
    let date_part = date.split_whitespace().next().unwrap_or(date);

    // Already in correct format
    if date_part.len() == 10 && date_part.chars().nth(4) == Some('-') {
        return date_part.to_string();
    }

    date_part.to_string()
}
