use super::state::App;
use super::tree::{FlattenedTree, TreeNode};

pub fn flattened(app: &App) -> FlattenedTree {
    FlattenedTree::from_root(&app.root)
}

pub fn move_down(app: &mut App) {
    let flat = flattened(app);
    if app.cursor.selected < flat.nodes.len().saturating_sub(1) {
        app.cursor.selected += 1;
        app.cursor.reset_cursor();
    }
}

pub fn move_up(app: &mut App) {
    if app.cursor.selected > 0 {
        app.cursor.selected -= 1;
        app.cursor.reset_cursor();
    }
}

pub fn current_word_len(app: &App) -> usize {
    let flat = flattened(app);
    if let Some(node) = flat.nodes.get(app.cursor.selected) {
        if app.cursor.cursor_on_value {
            node.value_preview.chars().count()
        } else {
            node.key.chars().count()
        }
    } else {
        0
    }
}

pub fn move_left(app: &mut App) {
    if app.cursor.cursor_pos > 0 {
        app.cursor.cursor_pos -= 1;
    } else if app.cursor.cursor_on_value {
        app.cursor.cursor_on_value = false;
        let flat = flattened(app);
        if let Some(node) = flat.nodes.get(app.cursor.selected) {
            app.cursor.cursor_pos = node.key.chars().count().saturating_sub(1);
        }
    } else {
        collapse_current(app);
    }
}

pub fn move_right(app: &mut App) {
    let word_len = current_word_len(app);
    if app.cursor.cursor_pos < word_len.saturating_sub(1) {
        app.cursor.cursor_pos += 1;
    } else if !app.cursor.cursor_on_value {
        app.cursor.cursor_on_value = true;
        app.cursor.cursor_pos = 0;
    } else {
        expand_current(app);
    }
}

pub fn word_forward(app: &mut App) {
    if !app.cursor.cursor_on_value {
        app.cursor.cursor_on_value = true;
        app.cursor.cursor_pos = 0;
    } else {
        move_down(app);
        app.cursor.cursor_on_value = false;
        app.cursor.cursor_pos = 0;
    }
}

pub fn word_backward(app: &mut App) {
    if app.cursor.cursor_on_value {
        app.cursor.cursor_on_value = false;
        app.cursor.cursor_pos = 0;
    } else {
        move_up(app);
        app.cursor.cursor_on_value = true;
        app.cursor.cursor_pos = 0;
    }
}

pub fn go_to_top(app: &mut App) {
    app.cursor.selected = 0;
}

pub fn go_to_bottom(app: &mut App) {
    let flat = flattened(app);
    app.cursor.selected = flat.nodes.len().saturating_sub(1);
}

pub fn page_down(app: &mut App, page_size: usize) {
    let flat = flattened(app);
    app.cursor.selected = (app.cursor.selected + page_size).min(flat.nodes.len().saturating_sub(1));
}

pub fn page_up(app: &mut App, page_size: usize) {
    app.cursor.selected = app.cursor.selected.saturating_sub(page_size);
}

pub fn toggle_expand(app: &mut App) {
    let flat = flattened(app);
    if let Some(node) = flat.nodes.get(app.cursor.selected) {
        if node.expandable {
            let path = node.path.clone();
            toggle_at_path(app, &path);
        }
    }
}

fn toggle_at_path(app: &mut App, path: &[usize]) {
    let node = navigate_mut(&mut app.root, path);
    node.expanded = !node.expanded;
}

pub fn expand_current(app: &mut App) {
    let flat = flattened(app);
    if let Some(node) = flat.nodes.get(app.cursor.selected) {
        if node.expandable && !node.expanded {
            let path = node.path.clone();
            set_expanded_at_path(app, &path, true);
        }
    }
}

pub fn collapse_current(app: &mut App) {
    let flat = flattened(app);
    if let Some(node) = flat.nodes.get(app.cursor.selected) {
        if node.expandable && node.expanded {
            let path = node.path.clone();
            set_expanded_at_path(app, &path, false);
        } else if node.depth > 0 {
            go_to_parent(app);
        }
    }
}

fn set_expanded_at_path(app: &mut App, path: &[usize], expanded: bool) {
    let node = navigate_mut(&mut app.root, path);
    node.expanded = expanded;
}

fn go_to_parent(app: &mut App) {
    let flat = flattened(app);
    if let Some(node) = flat.nodes.get(app.cursor.selected) {
        if !node.path.is_empty() {
            let parent_path: Vec<usize> = node.path[..node.path.len() - 1].to_vec();
            for (i, n) in flat.nodes.iter().enumerate() {
                if n.path == parent_path {
                    app.cursor.selected = i;
                    break;
                }
            }
        }
    }
}

pub fn navigate_mut<'b>(root: &'b mut TreeNode, path: &[usize]) -> &'b mut TreeNode {
    let mut current = root;
    for &idx in path {
        current = &mut current.children_mut().unwrap()[idx];
    }
    current
}

pub fn get_current_path(app: &App) -> String {
    let flat = flattened(app);
    if let Some(node) = flat.nodes.get(app.cursor.selected) {
        let mut parts = vec![app.root.key.clone()];
        let mut current = &app.root;
        for &idx in &node.path {
            if let Some(children) = current.children() {
                current = &children[idx];
                parts.push(current.key.clone());
            }
        }
        parts[1..].join(".")
    } else {
        String::new()
    }
}
