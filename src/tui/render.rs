use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use super::navigation::get_current_path;
use super::state::{App, UiState};
use super::tree::FlattenedTree;
use crate::settings::InputMode;

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_tree(f, app, chunks[2]);
    render_footer(f, app, chunks[4]);

    if app.show_help {
        render_help(f, app.input_mode);
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let path = get_current_path(app);
    let path_display = if path.is_empty() { "(root)" } else { &path };
    let brand = " prefer ";

    let dirty_indicator = if app.dirty { " [+]" } else { "" };

    let available_width = area.width as usize;
    let brand_len = brand.len();
    let path_len = path_display.len();
    let dirty_len = dirty_indicator.len();
    let file_max_len = available_width.saturating_sub(path_len + brand_len + dirty_len + 4);

    let file_display = if app.file_path.len() > file_max_len {
        format!(
            "…{}",
            &app.file_path[app.file_path.len().saturating_sub(file_max_len - 1)..]
        )
    } else {
        app.file_path.clone()
    };

    let padding =
        available_width.saturating_sub(file_display.len() + dirty_len + path_len + brand_len + 2);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            &file_display,
            Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        ),
        Span::styled(dirty_indicator, Style::default().fg(Color::Yellow)),
        Span::raw(" ".repeat(padding)),
        Span::styled(path_display, Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(brand, Style::default().fg(Color::Black).bg(Color::White)),
    ]));

    f.render_widget(header, area);
}

fn render_tree(f: &mut Frame, app: &App, area: Rect) {
    let flat = FlattenedTree::from_root(&app.root);
    let visible_height = area.height as usize;

    let scroll_offset = calculate_scroll(app.cursor.selected, app.cursor.scroll_offset, visible_height);
    let is_editing = app.ui_state == UiState::Edit;

    let items: Vec<ListItem> = flat
        .nodes
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, node)| render_tree_node(app, i, node, is_editing))
        .collect();

    let list = List::new(items);
    f.render_widget(list, area);
}

fn calculate_scroll(selected: usize, current_offset: usize, visible_height: usize) -> usize {
    if selected >= current_offset + visible_height {
        selected.saturating_sub(visible_height - 1)
    } else if selected < current_offset {
        selected
    } else {
        current_offset
    }
}

fn render_tree_node<'a>(
    app: &App,
    index: usize,
    node: &super::tree::FlatNode,
    is_editing: bool,
) -> ListItem<'a> {
    let indent = "  ".repeat(node.depth);
    let expand_char = expand_indicator(node.expandable, node.expanded);

    let is_selected = index == app.cursor.selected;
    let is_search_match = app.search.results.contains(&index);
    let cursor_on_key = is_selected && !app.cursor.cursor_on_value;
    let cursor_on_val = is_selected && app.cursor.cursor_on_value;

    let key_style = node_key_style(cursor_on_key, is_search_match, is_selected);
    let type_style = Style::default().fg(Color::DarkGray);

    if is_selected && is_editing {
        render_editing_node(app, &indent, expand_char, node, type_style)
    } else {
        render_normal_node(
            app, &indent, expand_char, node, cursor_on_key, cursor_on_val,
            key_style, type_style,
        )
    }
}

fn expand_indicator(expandable: bool, expanded: bool) -> &'static str {
    if expandable {
        if expanded { "▼ " } else { "▶ " }
    } else {
        "  "
    }
}

fn node_key_style(cursor_on_key: bool, is_search_match: bool, is_selected: bool) -> Style {
    if cursor_on_key {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else if is_search_match {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    }
}

fn render_editing_node<'a>(
    app: &App,
    indent: &str,
    expand_char: &'static str,
    node: &super::tree::FlatNode,
    type_style: Style,
) -> ListItem<'a> {
    let edit_style = Style::default().fg(Color::Green);
    let cursor_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    let before = app.edit.buffer[..app.edit.cursor].to_string();
    let after = app.edit.buffer[app.edit.cursor..].to_string();

    let line = if app.edit.editing_key {
        Line::from(vec![
            Span::raw(indent.to_string()),
            Span::styled(expand_char, Style::default().fg(Color::DarkGray)),
            Span::styled(before, edit_style),
            Span::styled("│", cursor_style),
            Span::styled(after, edit_style),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::styled(node.value_preview.clone(), Style::default().fg(Color::Gray)),
            Span::styled(format!(" ({})", node.type_indicator), type_style),
        ])
    } else {
        Line::from(vec![
            Span::raw(indent.to_string()),
            Span::styled(expand_char, Style::default().fg(Color::DarkGray)),
            Span::styled(node.key.clone(), Style::default().fg(Color::Gray)),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::styled(before, edit_style),
            Span::styled("│", cursor_style),
            Span::styled(after, edit_style),
            Span::styled(format!(" ({})", node.type_indicator), type_style),
        ])
    };
    ListItem::new(line)
}

fn render_normal_node<'a>(
    app: &App,
    indent: &str,
    expand_char: &'static str,
    node: &super::tree::FlatNode,
    cursor_on_key: bool,
    cursor_on_val: bool,
    key_style: Style,
    type_style: Style,
) -> ListItem<'a> {
    let block_style = Style::default().fg(Color::Black).bg(Color::Cyan);
    let selected_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    let value_color = value_type_color(node.type_indicator);

    let line = if cursor_on_key {
        let (before, cursor, after) = split_at_cursor(&node.key, app.cursor.cursor_pos);
        Line::from(vec![
            Span::raw(indent.to_string()),
            Span::styled(expand_char, Style::default().fg(Color::DarkGray)),
            Span::styled(before, selected_style),
            Span::styled(cursor, block_style),
            Span::styled(after, selected_style),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::styled(node.value_preview.clone(), Style::default().fg(value_color)),
            Span::styled(format!(" ({})", node.type_indicator), type_style),
        ])
    } else if cursor_on_val {
        let (before, cursor, after) = split_at_cursor(&node.value_preview, app.cursor.cursor_pos);
        Line::from(vec![
            Span::raw(indent.to_string()),
            Span::styled(expand_char, Style::default().fg(Color::DarkGray)),
            Span::styled(node.key.clone(), key_style),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::styled(before, selected_style),
            Span::styled(cursor, block_style),
            Span::styled(after, selected_style),
            Span::styled(format!(" ({})", node.type_indicator), type_style),
        ])
    } else {
        Line::from(vec![
            Span::raw(indent.to_string()),
            Span::styled(expand_char, Style::default().fg(Color::DarkGray)),
            Span::styled(node.key.clone(), key_style),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::styled(node.value_preview.clone(), Style::default().fg(value_color)),
            Span::styled(format!(" ({})", node.type_indicator), type_style),
        ])
    };
    ListItem::new(line)
}

fn value_type_color(type_indicator: &str) -> Color {
    match type_indicator {
        "str" => Color::Green,
        "num" => Color::Yellow,
        "bool" => Color::Magenta,
        "null" => Color::DarkGray,
        _ => Color::Blue,
    }
}

fn split_at_cursor(s: &str, pos: usize) -> (String, String, String) {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return (String::new(), "█".to_string(), String::new());
    }
    let pos = pos.min(chars.len().saturating_sub(1));
    let before: String = chars[..pos].iter().collect();
    let cursor = chars[pos].to_string();
    let after: String = chars[pos + 1..].iter().collect();
    (before, cursor, after)
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let content = match app.ui_state {
        UiState::Command => Line::from(vec![
            Span::styled(&app.command_buffer, Style::default().fg(Color::Yellow)),
            Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
        ]),
        UiState::Edit => Line::from(Span::styled(
            "-- INSERT --",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        UiState::Normal => {
            if let Some(msg) = &app.message {
                Line::from(Span::styled(msg, Style::default().fg(Color::Yellow)))
            } else if let Some(op) = app.operator.pending_op {
                let pending = format!("{}{}", op, app.operator.motion);
                Line::from(Span::styled(pending, Style::default().fg(Color::Yellow)))
            } else {
                Line::from("")
            }
        }
    };

    let footer = Paragraph::new(content);
    f.render_widget(footer, area);
}

fn render_help(f: &mut Frame, input_mode: InputMode) {
    let area = centered_rect(60, 80, f.area());
    let help_items = build_help_lines(input_mode);

    let title = match input_mode {
        InputMode::Vi => " Help (vi mode) ",
        InputMode::Basic => " Help (basic mode) ",
    };

    let help = Paragraph::new(help_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(Clear, area);
    f.render_widget(help, area);
}

fn build_help_lines(input_mode: InputMode) -> Vec<Line<'static>> {
    let help_text = match input_mode {
        InputMode::Vi => vi_help_text(),
        InputMode::Basic => basic_help_text(),
    };

    help_text
        .iter()
        .map(|line| format_help_line(line))
        .collect()
}

fn format_help_line(line: &str) -> Line<'static> {
    if line.contains("──") {
        Line::from(Span::styled(line.to_string(), Style::default().fg(Color::DarkGray)))
    } else if line.starts_with("  ") && line.contains("  ") {
        let parts: Vec<&str> = line.splitn(2, "  ").filter(|s| !s.is_empty()).collect();
        if parts.len() == 2 {
            Line::from(vec![
                Span::styled(
                    format!("  {:12}", parts[0].trim()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(parts[1].to_string()),
            ])
        } else {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            ))
        }
    } else {
        Line::from(line.to_string())
    }
}

fn vi_help_text() -> Vec<&'static str> {
    vec![
        "",
        "  Navigation",
        "  ──────────────────────────────",
        "  h           Left (key) / collapse",
        "  j           Down",
        "  k           Up",
        "  l           Right (value) / expand",
        "  w / b       Next / prev word",
        "  Space       Toggle expand/collapse",
        "  g           Go to top",
        "  G           Go to bottom",
        "  Ctrl+d      Page down",
        "  Ctrl+u      Page up",
        "",
        "  Editing",
        "  ──────────────────────────────",
        "  i           Edit (insert mode)",
        "  a           Edit (append mode)",
        "  ciw         Change word under cursor",
        "  cip         Change entire value/key",
        "  diw         Delete word",
        "  dip         Clear value to null",
        "  dd          Delete entry",
        "  o           Add new key",
        "",
        "  Commands",
        "  ──────────────────────────────",
        "  /           Search",
        "  :w          Save",
        "  :q          Quit",
        "  :wq         Save and quit",
        "  n / N       Next / prev match",
        "  Esc         Clear / cancel",
        "",
        "  Other",
        "  ──────────────────────────────",
        "  ?           Toggle this help",
        "",
    ]
}

fn basic_help_text() -> Vec<&'static str> {
    vec![
        "",
        "  Navigation",
        "  ──────────────────────────────",
        "  ↓           Move down",
        "  ↑           Move up",
        "  ←           Collapse / go to parent",
        "  →           Expand",
        "  Enter       Edit value / toggle",
        "  Home        Go to top",
        "  End         Go to bottom",
        "  Page Down   Page down",
        "  Page Up     Page up",
        "",
        "  Editing",
        "  ──────────────────────────────",
        "  Enter       Edit selected value",
        "  Delete      Delete key",
        "  Insert      Add new key",
        "  Ctrl+S      Save",
        "",
        "  Search",
        "  ──────────────────────────────",
        "  Ctrl+F      Search",
        "  F3          Next match",
        "",
        "  Other",
        "  ──────────────────────────────",
        "  F1          Toggle this help",
        "  Esc         Quit",
        "",
    ]
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
