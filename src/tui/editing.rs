use super::navigation::{flattened, navigate_mut};
use super::state::{App, UiState};
use super::tree::NodeValue;

pub fn start_edit_value(app: &mut App, at_end: bool, clear: bool) {
    let flat = flattened(app);
    if let Some(node) = flat.nodes.get(app.cursor.selected) {
        if node.editable {
            let tree_node = navigate_mut(&mut app.root, &node.path);
            if let Some(val) = tree_node.editable_value() {
                let text = if clear { String::new() } else { val };
                app.edit.start(text, at_end && !clear, false);
                app.ui_state = UiState::Edit;
            }
        } else {
            app.message = Some("Cannot edit containers directly".to_string());
        }
    }
}

pub fn start_edit_key(app: &mut App, clear: bool) {
    let flat = flattened(app);
    if let Some(node) = flat.nodes.get(app.cursor.selected) {
        if node.path.is_empty() {
            app.message = Some("Cannot rename root".to_string());
            return;
        }
        let text = if clear { String::new() } else { node.key.clone() };
        app.edit.start(text, true, true);
        app.ui_state = UiState::Edit;
    }
}

fn find_word_bounds(s: &str, char_pos: usize) -> (usize, usize) {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return (0, 0);
    }
    let pos = char_pos.min(chars.len().saturating_sub(1));

    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

    let mut start = pos;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    let mut end = pos;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    if start == end && pos < chars.len() {
        end = pos + 1;
    }

    (start, end)
}

pub fn start_edit_word(app: &mut App, clear: bool) {
    let flat = flattened(app);
    let Some(node) = flat.nodes.get(app.cursor.selected) else { return };

    let (text, is_key, cursor_offset) = if app.cursor.cursor_on_value {
        if !node.editable {
            app.message = Some("Cannot edit containers directly".to_string());
            return;
        }
        let tree_node = navigate_mut(&mut app.root, &node.path);
        match tree_node.editable_value() {
            Some(val) => {
                let is_string = node.type_indicator == "str";
                let offset = if is_string { 1 } else { 0 };
                (val, false, offset)
            }
            None => return,
        }
    } else {
        if node.path.is_empty() {
            app.message = Some("Cannot rename root".to_string());
            return;
        }
        (node.key.clone(), true, 0)
    };

    let adjusted_pos = app.cursor.cursor_pos.saturating_sub(cursor_offset);
    let (start, end) = find_word_bounds(&text, adjusted_pos);
    let chars: Vec<char> = text.chars().collect();
    let word: String = chars[start..end].iter().collect();

    let word_text = if clear { String::new() } else { word };
    app.edit.start_word(text, word_text, (start, end), is_key);
    app.ui_state = UiState::Edit;
}

pub fn apply_edit(app: &mut App) {
    let flat = flattened(app);
    if let Some(node) = flat.nodes.get(app.cursor.selected) {
        let path = node.path.clone();
        let tree_node = navigate_mut(&mut app.root, &path);

        let final_value = app.edit.final_value();

        if app.edit.editing_key {
            tree_node.key = final_value;
            app.message = Some("Key renamed (unsaved)".to_string());
        } else {
            tree_node.set_value_from_string(&final_value);
            app.message = Some("Value updated (unsaved)".to_string());
        }
        app.dirty = true;
    }
    app.ui_state = UiState::Normal;
    app.edit.clear();
}

pub fn cancel_edit(app: &mut App) {
    app.ui_state = UiState::Normal;
    app.edit.clear();
    app.message = Some("Edit cancelled".to_string());
}

pub fn delete_word(app: &mut App) {
    let flat = flattened(app);
    let Some(node) = flat.nodes.get(app.cursor.selected) else { return };

    let (text, cursor_offset) = if app.cursor.cursor_on_value {
        if !node.editable {
            app.message = Some("Cannot edit containers".to_string());
            return;
        }
        let tree_node = navigate_mut(&mut app.root, &node.path);
        match tree_node.editable_value() {
            Some(val) => {
                let is_string = node.type_indicator == "str";
                let offset = if is_string { 1 } else { 0 };
                (val, offset)
            }
            None => return,
        }
    } else {
        app.message = Some("Cannot delete part of key".to_string());
        return;
    };

    let adjusted_pos = app.cursor.cursor_pos.saturating_sub(cursor_offset);
    let (start, end) = find_word_bounds(&text, adjusted_pos);
    let chars: Vec<char> = text.chars().collect();
    let before: String = chars[..start].iter().collect();
    let after: String = chars[end..].iter().collect();
    let new_value = format!("{}{}", before, after).trim().to_string();

    let path = node.path.clone();
    let tree_node = navigate_mut(&mut app.root, &path);
    tree_node.set_value_from_string(&new_value);
    app.dirty = true;
    app.message = Some("Word deleted (unsaved)".to_string());
}

pub fn clear_value(app: &mut App) {
    let flat = flattened(app);
    if let Some(node) = flat.nodes.get(app.cursor.selected) {
        if node.editable {
            let tree_node = navigate_mut(&mut app.root, &node.path);
            tree_node.set_value_from_string("null");
            app.dirty = true;
            app.message = Some("Value cleared (unsaved)".to_string());
        } else {
            app.message = Some("Cannot clear containers".to_string());
        }
    }
}

pub fn delete_current(app: &mut App) {
    let flat = flattened(app);
    let Some(node) = flat.nodes.get(app.cursor.selected) else { return };

    if node.path.is_empty() {
        app.message = Some("Cannot delete root".to_string());
        return;
    }

    let parent_path = &node.path[..node.path.len() - 1];
    let child_index = *node.path.last().unwrap();

    let parent = navigate_mut(&mut app.root, parent_path);
    if parent.remove_child(child_index).is_some() {
        app.dirty = true;
        app.message = Some("Deleted (unsaved)".to_string());

        let new_flat = flattened(app);
        if app.cursor.selected >= new_flat.nodes.len() {
            app.cursor.selected = new_flat.nodes.len().saturating_sub(1);
        }
    }
}

pub fn add_new_key(app: &mut App) {
    let flat = flattened(app);
    let Some(node) = flat.nodes.get(app.cursor.selected) else { return };

    let path = node.path.clone();
    let target = navigate_mut(&mut app.root, &path);

    if target.is_expandable() {
        target.expanded = true;
        let is_array = matches!(target.value, NodeValue::Array(_));
        let key = if is_array {
            String::new()
        } else {
            "new_key".to_string()
        };
        target.add_child(key, NodeValue::String("value".to_string()));
        app.dirty = true;
        app.message = Some("Added new key (unsaved)".to_string());
    } else {
        app.message = Some("Can only add to objects/arrays".to_string());
    }
}

pub fn save(app: &mut App) -> anyhow::Result<()> {
    let config = app.root.to_config_value();
    if let prefer::ConfigValue::Object(obj) = config {
        if let Some(inner) = obj.into_iter().next() {
            app.backend.set(&app.resolved_path, "", &format_config_value(&inner.1))?;
        }
    }
    app.dirty = false;
    app.message = Some("Saved".to_string());
    Ok(())
}

fn format_config_value(value: &prefer::ConfigValue) -> String {
    match value {
        prefer::ConfigValue::Null => "null".to_string(),
        prefer::ConfigValue::Bool(b) => b.to_string(),
        prefer::ConfigValue::Integer(n) => n.to_string(),
        prefer::ConfigValue::Float(f) => f.to_string(),
        prefer::ConfigValue::String(s) => s.clone(),
        _ => String::new(),
    }
}
