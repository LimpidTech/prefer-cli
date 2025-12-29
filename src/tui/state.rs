use crate::backend::ConfigBackend;
use crate::settings::InputMode;
use prefer::ConfigValue;
use std::path::PathBuf;

use super::tree::TreeNode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiState {
    Normal,
    Command,
    Edit,
}

#[derive(Debug, Clone, Default)]
pub struct CursorState {
    pub selected: usize,
    pub cursor_on_value: bool,
    pub cursor_pos: usize,
    pub scroll_offset: usize,
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            cursor_on_value: true,
            ..Default::default()
        }
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_pos = 0;
    }
}

#[derive(Debug, Clone, Default)]
pub struct EditState {
    pub buffer: String,
    pub cursor: usize,
    pub editing_key: bool,
    pub word_range: Option<(usize, usize)>,
    pub original: String,
}

impl EditState {
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.original.clear();
        self.word_range = None;
        self.editing_key = false;
        self.cursor = 0;
    }

    pub fn start(&mut self, text: String, at_end: bool, is_key: bool) {
        self.buffer = text;
        self.cursor = if at_end { self.buffer.len() } else { 0 };
        self.editing_key = is_key;
        self.word_range = None;
        self.original.clear();
    }

    pub fn start_word(&mut self, original: String, word: String, range: (usize, usize), is_key: bool) {
        self.original = original;
        self.buffer = word;
        self.word_range = Some(range);
        self.cursor = 0;
        self.editing_key = is_key;
    }

    pub fn final_value(&self) -> String {
        if let Some((start, end)) = self.word_range {
            let chars: Vec<char> = self.original.chars().collect();
            let before: String = chars[..start].iter().collect();
            let after: String = chars[end..].iter().collect();
            format!("{}{}{}", before, self.buffer, after)
        } else {
            self.buffer.clone()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub results: Vec<usize>,
    pub index: usize,
}

impl SearchState {
    pub fn clear(&mut self) {
        self.results.clear();
        self.index = 0;
    }

    pub fn next(&mut self) -> Option<usize> {
        if self.results.is_empty() {
            return None;
        }
        self.index = (self.index + 1) % self.results.len();
        Some(self.results[self.index])
    }

    pub fn prev(&mut self) -> Option<usize> {
        if self.results.is_empty() {
            return None;
        }
        self.index = if self.index == 0 {
            self.results.len() - 1
        } else {
            self.index - 1
        };
        Some(self.results[self.index])
    }
}

#[derive(Debug, Clone, Default)]
pub struct OperatorState {
    pub pending_op: Option<char>,
    pub motion: String,
}

impl OperatorState {
    pub fn set(&mut self, op: char) {
        self.pending_op = Some(op);
        self.motion.clear();
    }

    pub fn clear(&mut self) {
        self.pending_op = None;
        self.motion.clear();
    }

    pub fn push_motion(&mut self, c: char) {
        self.motion.push(c);
    }
}

pub struct App<'a> {
    pub root: TreeNode,
    pub cursor: CursorState,
    pub edit: EditState,
    pub search: SearchState,
    pub operator: OperatorState,
    pub ui_state: UiState,
    pub command_buffer: String,
    pub file_path: String,
    pub resolved_path: PathBuf,
    pub show_help: bool,
    pub message: Option<String>,
    pub input_mode: InputMode,
    pub dirty: bool,
    pub backend: &'a dyn ConfigBackend,
}

impl<'a> App<'a> {
    pub fn new(
        config: ConfigValue,
        file_path: String,
        resolved_path: PathBuf,
        input_mode: InputMode,
        backend: &'a dyn ConfigBackend,
    ) -> Self {
        let root = TreeNode::from_config_value("root".to_string(), &config, 0);
        Self {
            root,
            cursor: CursorState::new(),
            edit: EditState::default(),
            search: SearchState::default(),
            operator: OperatorState::default(),
            ui_state: UiState::Normal,
            command_buffer: String::new(),
            file_path,
            resolved_path,
            show_help: false,
            message: None,
            input_mode,
            dirty: false,
            backend,
        }
    }
}
