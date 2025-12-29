use crossterm::event::{KeyCode, KeyModifiers};

use super::editing::{
    add_new_key, apply_edit, cancel_edit, clear_value, delete_current, delete_word, save,
    start_edit_key, start_edit_value, start_edit_word,
};
use super::navigation::{
    collapse_current, expand_current, flattened, go_to_bottom, go_to_top, move_down, move_left,
    move_right, move_up, page_down, page_up, toggle_expand, word_backward, word_forward,
};
use super::state::{App, UiState};
use crate::settings::InputMode;

pub fn handle_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> bool {
    match app.ui_state {
        UiState::Normal => match app.input_mode {
            InputMode::Vi => handle_vi_normal(app, code, modifiers),
            InputMode::Basic => handle_basic_normal(app, code, modifiers),
        },
        UiState::Command => {
            handle_command_input(app, code);
            false
        }
        UiState::Edit => {
            handle_edit_input(app, code);
            false
        }
    }
}

fn handle_command_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.ui_state = UiState::Normal;
            app.command_buffer.clear();
        }
        KeyCode::Backspace => {
            if app.command_buffer.len() > 1 {
                app.command_buffer.pop();
            } else {
                app.ui_state = UiState::Normal;
                app.command_buffer.clear();
            }
        }
        KeyCode::Char(c) => {
            app.command_buffer.push(c);
        }
        _ => {}
    }
}

fn handle_edit_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => apply_edit(app),
        KeyCode::Esc => cancel_edit(app),
        KeyCode::Backspace => {
            if app.edit.cursor > 0 {
                app.edit.cursor -= 1;
                app.edit.buffer.remove(app.edit.cursor);
            }
        }
        KeyCode::Delete => {
            if app.edit.cursor < app.edit.buffer.len() {
                app.edit.buffer.remove(app.edit.cursor);
            }
        }
        KeyCode::Left => {
            if app.edit.cursor > 0 {
                app.edit.cursor -= 1;
            }
        }
        KeyCode::Right => {
            if app.edit.cursor < app.edit.buffer.len() {
                app.edit.cursor += 1;
            }
        }
        KeyCode::Home => app.edit.cursor = 0,
        KeyCode::End => app.edit.cursor = app.edit.buffer.len(),
        KeyCode::Char(c) => {
            app.edit.buffer.insert(app.edit.cursor, c);
            app.edit.cursor += 1;
        }
        _ => {}
    }
}

fn handle_vi_normal(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> bool {
    if let Some(op) = app.operator.pending_op {
        return handle_pending_operator(app, op, code);
    }

    match code {
        KeyCode::Char('j') => move_down(app),
        KeyCode::Char('k') => move_up(app),
        KeyCode::Char('l') => move_right(app),
        KeyCode::Char('h') => move_left(app),
        KeyCode::Char('w') => word_forward(app),
        KeyCode::Char('b') => word_backward(app),
        KeyCode::Char(' ') => toggle_expand(app),
        KeyCode::Char('g') => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                go_to_bottom(app);
            } else {
                go_to_top(app);
            }
        }
        KeyCode::Char('G') => go_to_bottom(app),
        KeyCode::Char('/') => start_command(app, '/'),
        KeyCode::Char(':') => start_command(app, ':'),
        KeyCode::Char('n') => next_search_result(app),
        KeyCode::Char('N') => prev_search_result(app),
        KeyCode::Char('?') => app.show_help = !app.show_help,
        KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => page_down(app, 10),
        KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => page_up(app, 10),
        KeyCode::Char('i') => start_edit_value(app, false, false),
        KeyCode::Char('a') => start_edit_value(app, true, false),
        KeyCode::Char('c') | KeyCode::Char('d') => {
            if let KeyCode::Char(c) = code {
                app.operator.set(c);
            }
        }
        KeyCode::Char('o') => add_new_key(app),
        KeyCode::Esc => {
            app.show_help = false;
            app.search.clear();
            app.operator.clear();
        }
        _ => {}
    }
    false
}

fn handle_pending_operator(app: &mut App, op: char, code: KeyCode) -> bool {
    if let KeyCode::Char(c) = code {
        app.operator.push_motion(c);
        let motion = app.operator.motion.clone();

        let complete = matches!(
            motion.as_str(),
            "d" | "w" | "p" | "iw" | "aw" | "ip" | "ap"
        );

        if complete {
            execute_operator(app, op, &motion);
            app.operator.clear();
        } else if motion.len() >= 2 {
            app.message = Some(format!("Unknown motion: {}", motion));
            app.operator.clear();
        }
    } else if code == KeyCode::Esc {
        app.operator.clear();
    }
    false
}

fn handle_basic_normal(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> bool {
    match code {
        KeyCode::Esc => {
            if app.dirty {
                app.message = Some("Unsaved changes! Press Esc again or Ctrl+S to save".to_string());
                return false;
            }
            return true;
        }
        KeyCode::Down => move_down(app),
        KeyCode::Up => move_up(app),
        KeyCode::Right => expand_current(app),
        KeyCode::Left => collapse_current(app),
        KeyCode::Enter => {
            let flat = flattened(app);
            if let Some(node) = flat.nodes.get(app.cursor.selected) {
                if node.editable {
                    start_edit_value(app, true, false);
                } else {
                    toggle_expand(app);
                }
            }
        }
        KeyCode::Home => go_to_top(app),
        KeyCode::End => go_to_bottom(app),
        KeyCode::PageDown => page_down(app, 10),
        KeyCode::PageUp => page_up(app, 10),
        KeyCode::Char('f') if modifiers.contains(KeyModifiers::CONTROL) => start_command(app, '/'),
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(e) = save(app) {
                app.message = Some(format!("Save failed: {}", e));
            }
        }
        KeyCode::Delete => delete_current(app),
        KeyCode::Insert => add_new_key(app),
        KeyCode::F(3) => next_search_result(app),
        KeyCode::F(1) => app.show_help = !app.show_help,
        _ => {}
    }
    false
}

pub fn execute_command(app: &mut App) -> bool {
    let mut should_quit = false;

    if app.command_buffer.starts_with('/') {
        app.search.query = app.command_buffer[1..].to_string();
        execute_search(app);
    } else if app.command_buffer.starts_with(':') {
        let cmd = &app.command_buffer[1..].to_string();
        match cmd.as_str() {
            "q" | "quit" => {
                if app.dirty {
                    app.message = Some("Unsaved changes! Use :q! to force or :wq to save".to_string());
                } else {
                    should_quit = true;
                }
            }
            "q!" => should_quit = true,
            "w" | "write" => {
                if let Err(e) = save(app) {
                    app.message = Some(format!("Save failed: {}", e));
                }
            }
            "wq" | "x" => {
                if let Err(e) = save(app) {
                    app.message = Some(format!("Save failed: {}", e));
                } else {
                    should_quit = true;
                }
            }
            _ => app.message = Some(format!("Unknown command: {}", cmd)),
        }
    }
    app.ui_state = UiState::Normal;
    should_quit
}

fn start_command(app: &mut App, prefix: char) {
    app.ui_state = UiState::Command;
    app.command_buffer.clear();
    app.command_buffer.push(prefix);
}

fn execute_search(app: &mut App) {
    app.search.clear();
    if app.search.query.is_empty() {
        return;
    }

    let query = app.search.query.to_lowercase();
    let flat = flattened(app);

    for (i, node) in flat.nodes.iter().enumerate() {
        if node.key.to_lowercase().contains(&query)
            || node.value_preview.to_lowercase().contains(&query)
        {
            app.search.results.push(i);
        }
    }

    if !app.search.results.is_empty() {
        app.cursor.selected = app.search.results[0];
        app.message = Some(format!("Found {} match(es)", app.search.results.len()));
    } else {
        app.message = Some("No matches found".to_string());
    }
}

fn next_search_result(app: &mut App) {
    if let Some(idx) = app.search.next() {
        app.cursor.selected = idx;
    }
}

fn prev_search_result(app: &mut App) {
    if let Some(idx) = app.search.prev() {
        app.cursor.selected = idx;
    }
}

fn execute_operator(app: &mut App, op: char, motion: &str) {
    match (op, motion) {
        ('c', "iw" | "aw" | "w") => start_edit_word(app, true),
        ('c', "ip" | "ap" | "p") => {
            if app.cursor.cursor_on_value {
                start_edit_value(app, false, true);
            } else {
                start_edit_key(app, true);
            }
        }
        ('d', "d") => delete_current(app),
        ('d', "iw" | "aw" | "w") => delete_word(app),
        ('d', "ip" | "ap" | "p") => {
            if app.cursor.cursor_on_value {
                clear_value(app);
            } else {
                app.message = Some("Use dd to delete entry".to_string());
            }
        }
        _ => app.message = Some(format!("Unknown: {}{}", op, motion)),
    }
}
