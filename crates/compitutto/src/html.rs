use anyhow::Result;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::types::HomeworkEntry;

/// Generate HTML file from homework entries
pub fn generate_html(entries: &[HomeworkEntry], path: &Path) -> Result<()> {
    let html = render_page(entries);
    fs::write(path, html.into_string())?;
    Ok(())
}

pub fn render_page(entries: &[HomeworkEntry]) -> Markup {
    // Group entries by date
    let mut by_date: BTreeMap<&str, Vec<&HomeworkEntry>> = BTreeMap::new();
    for entry in entries {
        by_date.entry(&entry.date).or_default().push(entry);
    }

    let total_count = entries.len();

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Compitutto" }
                style { (PreEscaped(CSS)) }
            }
            body {
                div.container {
                    h1 { "Compitutto" }
                    div.stats {
                        span #"total-count" { (total_count) }
                        " entries"
                    }
                    div.homework-list #"homework-list" {
                        @if entries.is_empty() {
                            div.empty-state {
                                p { "No homework entries found." }
                            }
                        } @else {
                            @for (idx, (date, items)) in by_date.iter().enumerate() {
                                (render_date_group(date, items, idx))
                            }
                        }
                    }
                }
                script { (PreEscaped(JAVASCRIPT)) }
            }
        }
    }
}

fn render_date_group(date: &str, items: &[&HomeworkEntry], group_idx: usize) -> Markup {
    html! {
        div.date-group {
            div.date-header { "📅 " (date) }
            @for (item_idx, item) in items.iter().enumerate() {
                @let entry_id = group_idx * 100 + item_idx + 1;
                div.homework-item data-entry-id=(entry_id) {
                    input.homework-checkbox type="checkbox" id={"entry-" (entry_id)} data-entry-id=(entry_id);
                    div.homework-content {
                        div.homework-subject {
                            (item.subject)
                            @if !item.entry_type.is_empty() {
                                span.homework-type { (item.entry_type) }
                            }
                        }
                        div.homework-task { (item.task) }
                    }
                }
            }
        }
    }
}

const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;700;900&display=swap');

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    background: #0a0a0a;
    color: #fff;
    min-height: 100vh;
    padding: 0;
    line-height: 1.4;
    overflow-x: hidden;
}

body::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: 
        repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(255,255,255,0.03) 2px, rgba(255,255,255,0.03) 4px),
        radial-gradient(circle at 20% 50%, rgba(255,0,150,0.1) 0%, transparent 50%),
        radial-gradient(circle at 80% 80%, rgba(0,255,255,0.1) 0%, transparent 50%);
    pointer-events: none;
    z-index: 0;
}

.container {
    max-width: 1000px;
    margin: 0 auto;
    padding: 40px 24px 60px;
    position: relative;
    z-index: 1;
}

h1 {
    color: #fff;
    font-weight: 900;
    font-size: 4.5em;
    letter-spacing: -0.03em;
    margin-bottom: 4px;
    text-transform: uppercase;
    text-shadow: 
        0 0 10px rgba(255,0,150,0.5),
        0 0 20px rgba(0,255,255,0.3),
        4px 4px 0 #ff0096,
        -2px -2px 0 #00ffff;
    transform: rotate(-1deg);
    animation: glitch 3s infinite;
}

@keyframes glitch {
    0%, 100% { transform: rotate(-1deg) translate(0, 0); }
    25% { transform: rotate(-0.5deg) translate(-1px, 1px); }
    50% { transform: rotate(-1.5deg) translate(1px, -1px); }
    75% { transform: rotate(-0.8deg) translate(-1px, -1px); }
}

.stats {
    color: #888;
    font-size: 0.85em;
    font-weight: 700;
    margin-bottom: 50px;
    padding-top: 8px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
}

.homework-list {
    display: grid;
    gap: 50px;
}

.date-group {
    border-left: 4px solid;
    border-image: linear-gradient(180deg, #ff0096, #00ffff) 1;
    padding-left: 28px;
    margin-left: 4px;
    position: relative;
}

.date-group::before {
    content: '';
    position: absolute;
    left: -2px;
    top: 0;
    width: 8px;
    height: 8px;
    background: #00ffff;
    box-shadow: 0 0 10px #00ffff;
    border-radius: 50%;
}

.date-header {
    color: #fff;
    font-weight: 900;
    font-size: 1.1em;
    text-transform: uppercase;
    letter-spacing: 0.15em;
    margin-bottom: 28px;
    padding-top: 4px;
    text-shadow: 0 0 8px rgba(0,255,255,0.6);
}

.homework-item {
    display: flex;
    align-items: flex-start;
    gap: 20px;
    padding: 20px;
    margin-bottom: 16px;
    background: rgba(255,255,255,0.03);
    border: 1px solid rgba(255,255,255,0.1);
    transition: all 0.2s;
    position: relative;
}

.homework-item::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    width: 3px;
    height: 100%;
    background: linear-gradient(180deg, #ff0096, #00ffff);
    opacity: 0;
    transition: opacity 0.2s;
}

.homework-item:hover {
    background: rgba(255,255,255,0.05);
    border-color: rgba(255,0,150,0.4);
    transform: translateX(4px);
}

.homework-item:hover::before {
    opacity: 1;
}

.homework-item:last-child {
    margin-bottom: 0;
}

.homework-item.completed {
    opacity: 0.3;
    filter: grayscale(1);
}

.homework-item.completed .homework-task {
    text-decoration: line-through;
}

.homework-checkbox {
    width: 24px;
    height: 24px;
    min-width: 24px;
    cursor: pointer;
    margin-top: 2px;
    accent-color: #ff0096;
    filter: drop-shadow(0 0 4px rgba(255,0,150,0.6));
}

.homework-content {
    flex: 1;
}

.homework-subject {
    color: #fff;
    font-weight: 700;
    font-size: 1.1em;
    margin-bottom: 8px;
    display: flex;
    align-items: center;
    gap: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.homework-type {
    display: inline-block;
    background: linear-gradient(135deg, #ff0096, #00ffff);
    color: #000;
    font-size: 0.65em;
    padding: 4px 10px;
    margin-left: 8px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    font-weight: 900;
    border: 1px solid #fff;
    box-shadow: 0 0 8px rgba(255,0,150,0.5);
}

.homework-task {
    color: #ccc;
    line-height: 1.6;
    font-size: 0.95em;
    margin-top: 4px;
}

.empty-state {
    padding: 60px 20px;
    text-align: center;
    color: #666;
    font-size: 0.9em;
}

@media (max-width: 768px) {
    h1 {
        font-size: 3em;
    }
    
    .container {
        padding: 30px 16px 40px;
    }
}
"#;

const JAVASCRIPT: &str = r#"
// Load saved checkbox states from localStorage
function loadCheckboxStates() {
    const saved = localStorage.getItem('homework-checkboxes');
    if (saved) {
        const states = JSON.parse(saved);
        Object.keys(states).forEach(entryId => {
            const checkbox = document.getElementById(`entry-${entryId}`);
            const item = document.querySelector(`[data-entry-id="${entryId}"]`);
            if (checkbox && states[entryId]) {
                checkbox.checked = true;
                if (item) item.classList.add('completed');
            }
        });
    }
}

// Save checkbox states to localStorage
function saveCheckboxState(entryId, checked) {
    const saved = localStorage.getItem('homework-checkboxes') || '{}';
    const states = JSON.parse(saved);
    states[entryId] = checked;
    localStorage.setItem('homework-checkboxes', JSON.stringify(states));
}

// Add event listeners to all checkboxes
document.querySelectorAll('.homework-checkbox').forEach(checkbox => {
    checkbox.addEventListener('change', function() {
        const entryId = this.getAttribute('data-entry-id');
        const item = document.querySelector(`[data-entry-id="${entryId}"]`);
        
        if (this.checked) {
            item.classList.add('completed');
        } else {
            item.classList.remove('completed');
        }
        
        saveCheckboxState(entryId, this.checked);
    });
});

// Load states on page load
loadCheckboxStates();
"#;
