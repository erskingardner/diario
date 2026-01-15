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
                            @for (date, items) in by_date.iter() {
                                (render_date_group(date, items))
                            }
                        }
                    }
                }

                // Floating add button
                button.add-entry-btn #"add-entry-btn" type="button" title="Add new entry" { "+" }

                // Delete confirmation dialog
                dialog #"delete-dialog" {
                    h3 { "Delete Entry" }
                    p #"delete-message" { "Are you sure you want to delete this entry?" }
                    div.dialog-note #"delete-children-note" style="display:none" {
                        p { "This entry has study sessions linked to it." }
                        p { "Type " strong { "delete all" } " to delete everything, or " strong { "keep" } " to delete only this entry:" }
                        input #"delete-confirm-input" type="text" placeholder="Type here...";
                    }
                    div.dialog-buttons {
                        button.btn-cancel #"delete-cancel" type="button" { "Cancel" }
                        button.btn-danger #"delete-confirm" type="button" { "Delete" }
                    }
                }

                // Position dialog for drag-drop
                dialog #"position-dialog" {
                    h3 { "Position" }
                    p { "Where should this entry be placed?" }
                    div.dialog-buttons {
                        button.btn-primary #"position-top" type="button" { "Add to Top" }
                        button.btn-primary #"position-bottom" type="button" { "Add to Bottom" }
                        button.btn-cancel #"position-cancel" type="button" { "Cancel" }
                    }
                }

                // Add entry dialog
                dialog #"add-entry-dialog" {
                    h3 { "Add New Entry" }
                    form #"add-entry-form" {
                        div.form-group {
                            label for="new-entry-date" { "Date" }
                            input #"new-entry-date" type="date" required;
                        }
                        div.form-group {
                            label for="new-entry-subject" { "Subject" }
                            input #"new-entry-subject" type="text" placeholder="e.g., MATEMATICA" required;
                        }
                        div.form-group {
                            label for="new-entry-type" { "Type" }
                            select #"new-entry-type" {
                                option value="compiti" { "Compiti" }
                                option value="nota" { "Nota" }
                                option value="verifica" { "Verifica" }
                            }
                        }
                        div.form-group {
                            label for="new-entry-task" { "Task" }
                            textarea #"new-entry-task" rows="3" placeholder="Task description..." required {}
                        }
                        div.dialog-buttons {
                            button.btn-cancel #"add-entry-cancel" type="button" { "Cancel" }
                            button.btn-primary type="submit" { "Add Entry" }
                        }
                    }
                }

                script { (PreEscaped(JAVASCRIPT)) }
            }
        }
    }
}

fn render_date_group(date: &str, items: &[&HomeworkEntry]) -> Markup {
    html! {
        div.date-group data-date=(date) {
            div.date-header { "📅 " (date) }
            @for item in items.iter() {
                @let entry_id = &item.id;
                @let stable_id = item.stable_id();
                @let is_generated = item.is_generated();
                @let is_orphaned = item.is_orphaned();
                @let is_completed = item.completed;
                @let item_class = if is_completed { "homework-item completed" } else { "homework-item" };
                div
                    class=(item_class)
                    data-entry-id=(entry_id)
                    data-stable-id=(stable_id)
                    data-generated=[is_generated.then_some("true")]
                    data-orphaned=[is_orphaned.then_some("true")]
                    draggable="true"
                {
                    input.homework-checkbox
                        type="checkbox"
                        id={"entry-" (stable_id)}
                        data-entry-id=(entry_id)
                        checked[is_completed];
                    div.homework-content {
                        div.homework-subject {
                            (item.subject)
                            @if !item.entry_type.is_empty() {
                                span.homework-type { (item.entry_type) }
                            }
                            @if is_generated {
                                span.auto-badge { "auto" }
                            }
                            @if is_orphaned {
                                span.orphan-badge { "orphaned" }
                            }
                        }
                        div.homework-task { (item.task) }
                    }
                    button.delete-btn type="button" data-entry-id=(entry_id) title="Delete entry" { "🗑" }
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

/* Delete button */
.delete-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    background: transparent;
    border: none;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s;
    font-size: 14px;
    padding: 4px 8px;
    border-radius: 4px;
}

.homework-item:hover .delete-btn {
    opacity: 0.6;
}

.delete-btn:hover {
    opacity: 1 !important;
    background: rgba(255, 0, 0, 0.2);
}

/* Study session (generated) styling */
.homework-item[data-generated="true"] {
    border-left: 3px solid #00ffff;
    background: rgba(0, 255, 255, 0.03);
}

.homework-item[data-generated="true"]::before {
    display: none;
}

/* Orphaned study session */
.homework-item[data-orphaned="true"] {
    border-left: 3px dashed #ff9900;
    background: rgba(255, 153, 0, 0.03);
}

.homework-item[data-orphaned="true"]::before {
    display: none;
}

.auto-badge, .orphan-badge {
    font-size: 0.55em;
    padding: 2px 6px;
    border-radius: 3px;
    margin-left: 8px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.auto-badge {
    background: rgba(0, 255, 255, 0.2);
    color: #00ffff;
}

.orphan-badge {
    background: rgba(255, 153, 0, 0.2);
    color: #ff9900;
}

/* Drag states */
.homework-item.dragging {
    opacity: 0.4;
}

.homework-item[draggable="true"] {
    cursor: grab;
}

.homework-item[draggable="true"]:active {
    cursor: grabbing;
}

.date-group.drag-over {
    background: rgba(255, 0, 150, 0.05);
    box-shadow: inset 0 0 20px rgba(255, 0, 150, 0.2);
}

/* Add button */
.add-entry-btn {
    position: fixed;
    bottom: 30px;
    right: 30px;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: linear-gradient(135deg, #ff0096, #00ffff);
    border: none;
    color: #000;
    font-size: 28px;
    font-weight: bold;
    cursor: pointer;
    box-shadow: 0 4px 20px rgba(255, 0, 150, 0.4);
    z-index: 100;
    transition: transform 0.2s, box-shadow 0.2s;
}

.add-entry-btn:hover {
    transform: scale(1.1);
    box-shadow: 0 6px 30px rgba(255, 0, 150, 0.6);
}

/* Dialogs */
dialog {
    background: #1a1a1a;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    color: #fff;
    padding: 24px;
    max-width: 400px;
    width: 90%;
}

dialog::backdrop {
    background: rgba(0, 0, 0, 0.7);
}

dialog h3 {
    margin-bottom: 16px;
    font-size: 1.2em;
}

dialog p {
    margin-bottom: 12px;
    color: #ccc;
}

.dialog-note {
    background: rgba(255, 153, 0, 0.1);
    border: 1px solid rgba(255, 153, 0, 0.3);
    border-radius: 4px;
    padding: 12px;
    margin: 16px 0;
}

.dialog-note input {
    width: 100%;
    margin-top: 8px;
    padding: 8px;
    background: #0a0a0a;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
}

.dialog-buttons {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 20px;
}

.dialog-buttons button {
    padding: 10px 20px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-weight: 600;
    transition: all 0.2s;
}

.btn-cancel {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.3) !important;
    color: #fff;
}

.btn-cancel:hover {
    background: rgba(255, 255, 255, 0.1);
}

.btn-primary {
    background: linear-gradient(135deg, #ff0096, #00ffff);
    color: #000;
}

.btn-primary:hover {
    box-shadow: 0 0 15px rgba(255, 0, 150, 0.5);
}

.btn-danger {
    background: #ff3333;
    color: #fff;
}

.btn-danger:hover {
    background: #ff5555;
}

/* Form styles */
.form-group {
    margin-bottom: 16px;
}

.form-group label {
    display: block;
    margin-bottom: 6px;
    font-weight: 600;
    color: #ccc;
    font-size: 0.9em;
}

.form-group input,
.form-group select,
.form-group textarea {
    width: 100%;
    padding: 10px;
    background: #0a0a0a;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
    font-size: 1em;
}

.form-group input:focus,
.form-group select:focus,
.form-group textarea:focus {
    outline: none;
    border-color: #ff0096;
    box-shadow: 0 0 0 2px rgba(255, 0, 150, 0.2);
}

.form-group select {
    cursor: pointer;
}

.form-group textarea {
    resize: vertical;
    min-height: 80px;
}

@media (max-width: 768px) {
    h1 {
        font-size: 3em;
    }
    
    .container {
        padding: 30px 16px 40px;
    }
    
    .add-entry-btn {
        bottom: 20px;
        right: 20px;
        width: 48px;
        height: 48px;
        font-size: 24px;
    }
}
"#;

const JAVASCRIPT: &str = r#"
// ========== Checkbox Completion (API-backed) ==========

document.querySelectorAll('.homework-checkbox').forEach(checkbox => {
    checkbox.addEventListener('change', async function() {
        const entryId = this.getAttribute('data-entry-id');
        const item = document.querySelector(`[data-entry-id="${entryId}"]`);
        const isChecked = this.checked;
        
        // Optimistic UI update
        if (isChecked) {
            item.classList.add('completed');
        } else {
            item.classList.remove('completed');
        }
        
        // Persist to database via API
        try {
            const response = await fetch(`/api/entries/${entryId}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ completed: isChecked })
            });
            
            if (!response.ok) {
                // Revert on error
                this.checked = !isChecked;
                item.classList.toggle('completed');
                console.error('Failed to update completion state');
            }
        } catch (error) {
            // Revert on error
            this.checked = !isChecked;
            item.classList.toggle('completed');
            console.error('Error updating completion:', error);
        }
    });
});

// ========== Delete Functionality ==========

const deleteDialog = document.getElementById('delete-dialog');
const deleteMessage = document.getElementById('delete-message');
const deleteChildrenNote = document.getElementById('delete-children-note');
const deleteConfirmInput = document.getElementById('delete-confirm-input');
const deleteConfirmBtn = document.getElementById('delete-confirm');
const deleteCancelBtn = document.getElementById('delete-cancel');

let pendingDeleteId = null;
let pendingDeleteHasChildren = false;

document.querySelectorAll('.delete-btn').forEach(btn => {
    btn.addEventListener('click', async function(e) {
        e.stopPropagation();
        pendingDeleteId = this.getAttribute('data-entry-id');
        
        // Check if entry has children
        try {
            const response = await fetch(`/api/entries/${pendingDeleteId}/children`);
            const children = await response.json();
            pendingDeleteHasChildren = children.length > 0;
            
            if (pendingDeleteHasChildren) {
                deleteMessage.textContent = `This entry has ${children.length} study session(s) linked to it.`;
                deleteChildrenNote.style.display = 'block';
                deleteConfirmInput.value = '';
            } else {
                deleteMessage.textContent = 'Are you sure you want to delete this entry?';
                deleteChildrenNote.style.display = 'none';
            }
            
            deleteDialog.showModal();
        } catch (error) {
            console.error('Error checking children:', error);
        }
    });
});

deleteCancelBtn.addEventListener('click', () => {
    deleteDialog.close();
    pendingDeleteId = null;
    pendingDeleteHasChildren = false;
});

deleteConfirmBtn.addEventListener('click', async () => {
    if (!pendingDeleteId) return;
    
    if (pendingDeleteHasChildren) {
        const input = deleteConfirmInput.value.toLowerCase().trim();
        if (input !== 'delete all' && input !== 'keep') {
            deleteConfirmInput.focus();
            return;
        }
        
        try {
            if (input === 'delete all') {
                // Cascade delete
                await fetch(`/api/entries/${pendingDeleteId}/cascade`, { method: 'DELETE' });
            } else {
                // Delete only parent (orphans children)
                await fetch(`/api/entries/${pendingDeleteId}`, { method: 'DELETE' });
            }
            location.reload();
        } catch (error) {
            console.error('Delete error:', error);
        }
    } else {
        try {
            await fetch(`/api/entries/${pendingDeleteId}`, { method: 'DELETE' });
            location.reload();
        } catch (error) {
            console.error('Delete error:', error);
        }
    }
    
    deleteDialog.close();
});

// Close dialog on backdrop click
deleteDialog.addEventListener('click', (e) => {
    if (e.target === deleteDialog) deleteDialog.close();
});

// ========== Drag and Drop ==========

const positionDialog = document.getElementById('position-dialog');
const positionTopBtn = document.getElementById('position-top');
const positionBottomBtn = document.getElementById('position-bottom');
const positionCancelBtn = document.getElementById('position-cancel');

let draggedItem = null;
let draggedEntryId = null;
let targetDate = null;

document.querySelectorAll('.homework-item').forEach(item => {
    item.addEventListener('dragstart', function(e) {
        draggedItem = this;
        draggedEntryId = this.getAttribute('data-entry-id');
        this.classList.add('dragging');
        e.dataTransfer.effectAllowed = 'move';
    });
    
    item.addEventListener('dragend', function() {
        this.classList.remove('dragging');
        document.querySelectorAll('.date-group').forEach(g => g.classList.remove('drag-over'));
    });
});

document.querySelectorAll('.date-group').forEach(group => {
    group.addEventListener('dragover', function(e) {
        e.preventDefault();
        e.dataTransfer.dropEffect = 'move';
        this.classList.add('drag-over');
    });
    
    group.addEventListener('dragleave', function(e) {
        // Only remove if we're leaving the group entirely
        if (!this.contains(e.relatedTarget)) {
            this.classList.remove('drag-over');
        }
    });
    
    group.addEventListener('drop', function(e) {
        e.preventDefault();
        this.classList.remove('drag-over');
        
        if (!draggedItem) return;
        
        targetDate = this.getAttribute('data-date');
        const sourceDate = draggedItem.closest('.date-group').getAttribute('data-date');
        
        // If dropping on the same date, no need to ask for position
        if (targetDate === sourceDate) {
            draggedItem = null;
            return;
        }
        
        positionDialog.showModal();
    });
});

async function moveEntry(position) {
    if (!draggedEntryId || !targetDate) return;
    
    try {
        // Get max position for target date
        const entriesResponse = await fetch('/api/entries');
        const entries = await entriesResponse.json();
        const targetEntries = entries.filter(e => e.date === targetDate);
        
        let newPosition;
        if (position === 'top') {
            // Shift all existing entries down
            newPosition = 0;
            for (const entry of targetEntries) {
                await fetch(`/api/entries/${entry.id}`, {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ position: entry.position + 1 })
                });
            }
        } else {
            // Add to bottom
            newPosition = targetEntries.length > 0 
                ? Math.max(...targetEntries.map(e => e.position)) + 1 
                : 0;
        }
        
        // Update the entry with new date and position
        await fetch(`/api/entries/${draggedEntryId}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ date: targetDate, position: newPosition })
        });
        
        location.reload();
    } catch (error) {
        console.error('Error moving entry:', error);
    }
}

positionTopBtn.addEventListener('click', () => {
    positionDialog.close();
    moveEntry('top');
});

positionBottomBtn.addEventListener('click', () => {
    positionDialog.close();
    moveEntry('bottom');
});

positionCancelBtn.addEventListener('click', () => {
    positionDialog.close();
    draggedItem = null;
    draggedEntryId = null;
    targetDate = null;
});

positionDialog.addEventListener('click', (e) => {
    if (e.target === positionDialog) {
        positionDialog.close();
        draggedItem = null;
        draggedEntryId = null;
        targetDate = null;
    }
});

// ========== Add Entry ==========

const addEntryBtn = document.getElementById('add-entry-btn');
const addEntryDialog = document.getElementById('add-entry-dialog');
const addEntryForm = document.getElementById('add-entry-form');
const addEntryCancelBtn = document.getElementById('add-entry-cancel');

addEntryBtn.addEventListener('click', () => {
    // Set default date to today
    const today = new Date().toISOString().split('T')[0];
    document.getElementById('new-entry-date').value = today;
    document.getElementById('new-entry-subject').value = '';
    document.getElementById('new-entry-type').value = 'compiti';
    document.getElementById('new-entry-task').value = '';
    addEntryDialog.showModal();
});

addEntryCancelBtn.addEventListener('click', () => {
    addEntryDialog.close();
});

addEntryForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const entry = {
        date: document.getElementById('new-entry-date').value,
        subject: document.getElementById('new-entry-subject').value.toUpperCase(),
        entry_type: document.getElementById('new-entry-type').value,
        task: document.getElementById('new-entry-task').value
    };
    
    try {
        const response = await fetch('/api/entries', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(entry)
        });
        
        if (response.ok) {
            addEntryDialog.close();
            location.reload();
        } else {
            console.error('Failed to create entry');
        }
    } catch (error) {
        console.error('Error creating entry:', error);
    }
});

addEntryDialog.addEventListener('click', (e) => {
    if (e.target === addEntryDialog) addEntryDialog.close();
});
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a HomeworkEntry
    fn make_entry(entry_type: &str, date: &str, subject: &str, task: &str) -> HomeworkEntry {
        HomeworkEntry::new(
            entry_type.to_string(),
            date.to_string(),
            subject.to_string(),
            task.to_string(),
        )
    }

    // ========== render_page tests ==========

    #[test]
    fn test_render_page_empty_entries() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("Compitutto"));
        assert!(html.contains("No homework entries found"));
        assert!(html.contains("0")); // Total count
    }

    #[test]
    fn test_render_page_single_entry() {
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "MATEMATICA",
            "Pag. 100 es. 1-5",
        )];
        let html = render_page(&entries).into_string();

        assert!(html.contains("MATEMATICA"));
        assert!(html.contains("Pag. 100 es. 1-5"));
        assert!(html.contains("2025-01-15"));
        assert!(html.contains("compiti"));
        assert!(html.contains(">1<")); // Total count: 1
    }

    #[test]
    fn test_render_page_multiple_entries_same_date() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-15", "ITALIANO", "Task 2"),
        ];
        let html = render_page(&entries).into_string();

        assert!(html.contains("MATEMATICA"));
        assert!(html.contains("ITALIANO"));
        assert!(html.contains("Task 1"));
        assert!(html.contains("Task 2"));
        // Should only have one date-group element for 2025-01-15
        // (class="date-group" appears once in the HTML, once for each date group)
        assert_eq!(html.matches(r#"class="date-group""#).count(), 1);
    }

    #[test]
    fn test_render_page_multiple_dates() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),
            make_entry("compiti", "2025-01-17", "INGLESE", "Task 3"),
        ];
        let html = render_page(&entries).into_string();

        assert!(html.contains("2025-01-15"));
        assert!(html.contains("2025-01-16"));
        assert!(html.contains("2025-01-17"));
        assert!(html.contains(">3<")); // Total count: 3
    }

    #[test]
    fn test_render_page_has_required_elements() {
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let html = render_page(&entries).into_string();

        // Check document structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\""));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));

        // Check meta tags
        assert!(html.contains("charset=\"UTF-8\""));
        assert!(html.contains("viewport"));

        // Check title
        assert!(html.contains("<title>Compitutto</title>"));

        // Check CSS and JS
        assert!(html.contains("<style>"));
        assert!(html.contains("<script>"));
    }

    #[test]
    fn test_render_page_has_checkboxes() {
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let html = render_page(&entries).into_string();

        assert!(html.contains("homework-checkbox"));
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("data-entry-id"));
    }

    #[test]
    fn test_render_page_escapes_html_in_task() {
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "MATEMATICA",
            "<script>alert('xss')</script>",
        )];
        let html = render_page(&entries).into_string();

        // Should be escaped, not rendered as actual script tag
        assert!(!html.contains("<script>alert('xss')</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_render_page_handles_special_characters() {
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "MATEMATICA",
            "Esercizi con àèìòù & simboli",
        )];
        let html = render_page(&entries).into_string();

        // Ampersand should be escaped
        assert!(html.contains("Esercizi con àèìòù &amp; simboli"));
    }

    #[test]
    fn test_render_page_empty_entry_type() {
        let entries = vec![make_entry(
            "", // Empty type
            "2025-01-15",
            "MATEMATICA",
            "Task 1",
        )];
        let html = render_page(&entries).into_string();

        assert!(html.contains("MATEMATICA"));
        assert!(html.contains("Task 1"));
        // Should not have a type badge element (span with homework-type class)
        // The CSS class definition will still be there, but no <span class="homework-type">
        assert!(!html.contains("<span class=\"homework-type\">"));
    }

    #[test]
    fn test_render_page_entry_type_badge() {
        let entries = vec![make_entry("nota", "2025-01-15", "MATEMATICA", "Task 1")];
        let html = render_page(&entries).into_string();

        assert!(html.contains("homework-type"));
        assert!(html.contains("nota"));
    }

    #[test]
    fn test_render_page_groups_entries_by_date() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-15", "ITALIANO", "Task 2"),
            make_entry("compiti", "2025-01-16", "INGLESE", "Task 3"),
        ];
        let html = render_page(&entries).into_string();

        // Count date-group divs
        let date_groups = html.matches("date-group").count();
        // 2 groups (2 unique dates), each appearing in class name
        assert!(date_groups >= 2);
    }

    #[test]
    fn test_render_page_css_included() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        assert!(html.contains("font-family"));
        assert!(html.contains("background"));
        assert!(html.contains(".homework-item"));
        assert!(html.contains(".date-header"));
    }

    #[test]
    fn test_render_page_javascript_included() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        // JavaScript for checkbox handling (API-backed), drag-drop, delete, and add entry
        assert!(html.contains("homework-checkbox"));
        assert!(html.contains("/api/entries"));
        assert!(html.contains("dragstart"));
        assert!(html.contains("delete-dialog"));
    }

    // ========== render_date_group tests ==========

    #[test]
    fn test_render_date_group_basic() {
        let entries = [
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-15", "ITALIANO", "Task 2"),
        ];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let html = render_date_group("2025-01-15", &refs).into_string();

        assert!(html.contains("date-group"));
        assert!(html.contains("2025-01-15"));
        assert!(html.contains("MATEMATICA"));
        assert!(html.contains("ITALIANO"));
    }

    #[test]
    fn test_render_date_group_entry_ids_are_stable() {
        let entries = [
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-15", "ITALIANO", "Task 2"),
        ];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();

        let html = render_date_group("2025-01-15", &refs).into_string();

        // Entry IDs should be content-based hex strings
        let entry1_id = entries[0].stable_id();
        let entry2_id = entries[1].stable_id();

        assert!(html.contains(&format!("entry-{}", entry1_id)));
        assert!(html.contains(&format!("entry-{}", entry2_id)));

        // IDs should be 8-character hex strings
        assert_eq!(entry1_id.len(), 8);
        assert_eq!(entry2_id.len(), 8);
    }

    #[test]
    fn test_render_date_group_ids_independent_of_position() {
        let entry1 = make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1");
        let entry2 = make_entry("nota", "2025-01-16", "ITALIANO", "Task 2");

        // Render entry1 in first position
        let refs1: Vec<&HomeworkEntry> = vec![&entry1, &entry2];
        let html1 = render_date_group("2025-01-15", &refs1).into_string();

        // Render entry1 in second position
        let refs2: Vec<&HomeworkEntry> = vec![&entry2, &entry1];
        let html2 = render_date_group("2025-01-15", &refs2).into_string();

        // Entry1's ID should be the same regardless of position
        let entry1_id = entry1.stable_id();
        assert!(html1.contains(&format!("entry-{}", entry1_id)));
        assert!(html2.contains(&format!("entry-{}", entry1_id)));
    }

    // ========== generate_html tests ==========

    #[test]
    fn test_generate_html_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("index.html");

        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];

        generate_html(&entries, &html_path).unwrap();

        assert!(html_path.exists());
    }

    #[test]
    fn test_generate_html_content() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("index.html");

        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];

        generate_html(&entries, &html_path).unwrap();

        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("MATEMATICA"));
        assert!(content.contains("Task 1"));
    }

    #[test]
    fn test_generate_html_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("index.html");

        std::fs::write(&html_path, "old content").unwrap();

        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "MATEMATICA",
            "New task",
        )];

        generate_html(&entries, &html_path).unwrap();

        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(!content.contains("old content"));
        assert!(content.contains("New task"));
    }

    #[test]
    fn test_generate_html_empty_entries() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("index.html");

        generate_html(&[], &html_path).unwrap();

        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(content.contains("No homework entries found"));
    }

    // ========== Edge cases ==========

    #[test]
    fn test_render_page_long_task_text() {
        let long_task = "A".repeat(1000);
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "MATEMATICA",
            &long_task,
        )];
        let html = render_page(&entries).into_string();

        assert!(html.contains(&long_task));
    }

    #[test]
    fn test_render_page_unicode_content() {
        let entries = vec![make_entry("compiti", "2025-01-15", "日本語", "任务描述 🎉")];
        let html = render_page(&entries).into_string();

        assert!(html.contains("日本語"));
        assert!(html.contains("任务描述"));
    }

    #[test]
    fn test_render_page_many_entries() {
        let entries: Vec<HomeworkEntry> = (0..100)
            .map(|i| {
                make_entry(
                    "compiti",
                    &format!("2025-01-{:02}", (i % 28) + 1),
                    &format!("SUBJECT_{}", i),
                    &format!("Task {}", i),
                )
            })
            .collect();

        let html = render_page(&entries).into_string();

        assert!(html.contains(">100<")); // Total count
        assert!(html.contains("SUBJECT_0"));
        assert!(html.contains("SUBJECT_99"));
    }

    // ========== New feature tests ==========

    #[test]
    fn test_render_page_has_add_button() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        assert!(html.contains("add-entry-btn"));
        assert!(html.contains(r#"id="add-entry-btn""#));
    }

    #[test]
    fn test_render_page_has_delete_dialog() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        assert!(html.contains("delete-dialog"));
        assert!(html.contains("delete-confirm"));
        assert!(html.contains("delete-cancel"));
    }

    #[test]
    fn test_render_page_has_position_dialog() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        assert!(html.contains("position-dialog"));
        assert!(html.contains("position-top"));
        assert!(html.contains("position-bottom"));
    }

    #[test]
    fn test_render_page_has_add_entry_dialog() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        assert!(html.contains("add-entry-dialog"));
        assert!(html.contains("add-entry-form"));
        assert!(html.contains("new-entry-date"));
        assert!(html.contains("new-entry-subject"));
        assert!(html.contains("new-entry-type"));
        assert!(html.contains("new-entry-task"));
    }

    #[test]
    fn test_render_date_group_has_delete_buttons() {
        let entries = [make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let html = render_date_group("2025-01-15", &refs).into_string();

        assert!(html.contains("delete-btn"));
        assert!(html.contains(r#"title="Delete entry""#));
    }

    #[test]
    fn test_render_date_group_draggable() {
        let entries = [make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let html = render_date_group("2025-01-15", &refs).into_string();

        assert!(html.contains(r#"draggable="true""#));
    }

    #[test]
    fn test_render_date_group_data_date() {
        let entries = [make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let html = render_date_group("2025-01-15", &refs).into_string();

        assert!(html.contains(r#"data-date="2025-01-15""#));
    }

    #[test]
    fn test_render_date_group_generated_entry() {
        let mut entry = make_entry("studio", "2025-01-15", "MATEMATICA", "Study for: Test");
        entry.parent_id = Some("parent123".to_string());
        let refs: Vec<&HomeworkEntry> = vec![&entry];
        let html = render_date_group("2025-01-15", &refs).into_string();

        assert!(html.contains(r#"data-generated="true""#));
        assert!(html.contains("auto-badge"));
        assert!(html.contains("auto")); // badge text
    }

    #[test]
    fn test_render_date_group_orphaned_entry() {
        let entry = make_entry("studio", "2025-01-15", "MATEMATICA", "Study for: Test");
        // Note: entry_type is "studio" but parent_id is None, so is_orphaned() returns true
        let refs: Vec<&HomeworkEntry> = vec![&entry];
        let html = render_date_group("2025-01-15", &refs).into_string();

        assert!(html.contains(r#"data-orphaned="true""#));
        assert!(html.contains("orphan-badge"));
        assert!(html.contains("orphaned")); // badge text
    }

    #[test]
    fn test_render_date_group_completed_entry() {
        let mut entry = make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1");
        entry.completed = true;
        let refs: Vec<&HomeworkEntry> = vec![&entry];
        let html = render_date_group("2025-01-15", &refs).into_string();

        assert!(html.contains(r#"class="homework-item completed""#));
        assert!(html.contains("checked"));
    }

    #[test]
    fn test_render_page_css_has_generated_styling() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        assert!(html.contains("[data-generated=\"true\"]"));
        assert!(html.contains("auto-badge"));
    }

    #[test]
    fn test_render_page_css_has_orphaned_styling() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        assert!(html.contains("[data-orphaned=\"true\"]"));
        assert!(html.contains("orphan-badge"));
    }

    #[test]
    fn test_render_page_css_has_drag_styling() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        assert!(html.contains(".dragging"));
        assert!(html.contains(".drag-over"));
    }

    #[test]
    fn test_render_page_css_has_delete_button_styling() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();

        assert!(html.contains(".delete-btn"));
    }
}
