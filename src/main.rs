mod editor;
mod ui;
mod syntax;
mod lsp;
mod clipboard;
mod git;
mod search;
mod project;
mod config;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use editor::{Buffer, BufferPosition, BufferRange, Cursor, Selection, SelectionMode, TabManager, View};
use ui::Renderer;
use config::{KeybindingSet, Settings};
use clipboard::{ClipboardManager, ClipKind};
use git::GitIntegration;
use search::{SearchEngine, GlobalSearch};
use project::ProjectExplorer;
use lsp::LspClient;

const VERSION: &str = "1.0.0";
const AUTHOR: &str = "Ishmael Vertil";
const LICENSE: &str = "MIT";
const REPOSITORY: &str = "https://github.com/ishmaelvertil/vertil-nano-pro";

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    Insert,
    Search,
    Replace,
    Command,
    Explorer,
    GoToLine,
    SaveAs,
    OpenFile,
    ClipboardHistory,
    Help,
    GitStatus,
    About,
}

struct App {
    tab_manager: TabManager,
    view: View,
    cursor: Cursor,
    selection: Selection,
    mode: Mode,
    prev_mode: Mode,
    command_buffer: String,
    search_buffer: String,
    replace_buffer: String,
    message: Option<String>,
    lsp_client: LspClient,
    clipboard: ClipboardManager,
    git: GitIntegration,
    search_engine: SearchEngine,
    global_search: GlobalSearch,
    explorer: ProjectExplorer,
    keybindings: KeybindingSet,
    show_splash: bool,
    running: bool,
}

impl App {
    fn new(_settings: &Settings) -> Self {
        Self {
            tab_manager: TabManager::new(),
            view: View::new(80, 24),
            cursor: Cursor::new(),
            selection: Selection::new(),
            mode: Mode::Normal,
            prev_mode: Mode::Normal,
            command_buffer: String::new(),
            search_buffer: String::new(),
            replace_buffer: String::new(),
            message: None,
            lsp_client: LspClient::new(),
            clipboard: ClipboardManager::new(),
            git: GitIntegration::new(),
            search_engine: SearchEngine::new(),
            global_search: GlobalSearch::new(),
            explorer: ProjectExplorer::new(),
            keybindings: KeybindingSet::default(),
            show_splash: true,
            running: true,
        }
    }

    fn open_file(&mut self, path: PathBuf) {
        if let Err(e) = self.tab_manager.open_tab(Some(path.clone())) {
            self.message = Some(format!("Error: {}", e));
            return;
        }

        // Set up project root
        if self.explorer.root().is_none() {
            if let Some(root) = ProjectExplorer::find_project_root(&path) {
                self.explorer.set_root(&root);
            } else if let Some(parent) = path.parent() {
                self.explorer.set_root(parent);
            }
        }

        // Initialize git
        if !self.git.is_available() {
            if let Some(root) = self.explorer.root() {
                let _ = self.git.open(root);
            }
        }

        self.cursor = Cursor::new();
        self.selection.clear();
        self.message = Some(format!("Opened: {}", path.display()));
    }

    fn current_buffer(&self) -> Option<&Buffer> {
        self.tab_manager.active().map(|t| &t.buffer)
    }

    fn current_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.tab_manager.active_mut().map(|t| &mut t.buffer)
    }

    fn current_lines(&self) -> Vec<String> {
        self.current_buffer()
            .map(|b| b.lines().to_vec())
            .unwrap_or_default()
    }

    fn save_file(&mut self) {
        if let Some(tab) = self.tab_manager.active_mut() {
            match tab.buffer.save() {
                Ok(()) => {
                    self.message = Some(format!("Saved: {}", tab.buffer.filename()));
                }
                Err(e) => {
                    self.message = Some(format!("Error: {}", e));
                }
            }
        }
    }

    fn save_as(&mut self, path: &str) {
        let path = PathBuf::from(path);
        if let Some(tab) = self.tab_manager.active_mut() {
            match tab.buffer.save_to(&path) {
                Ok(()) => {
                    self.message = Some(format!("Saved as: {}", path.display()));
                    self.mode = Mode::Normal;
                }
                Err(e) => {
                    self.message = Some(format!("Error: {}", e));
                }
            }
        }
    }

    fn cut_selection_or_line(&mut self) {
        let lines = self.current_lines();
        if self.selection.active && !self.selection.is_empty() {
            let text = self.selection.get_selected_text(&lines);
            self.clipboard.copy(&text, ClipKind::Block);

            let start = self.selection.start();
            let end = self.selection.end();
            if let Some(buffer) = self.current_buffer_mut() {
                buffer.delete_range(BufferRange::new(start, end));
            }
            self.cursor.set_position(start);
            self.selection.clear();
        } else {
            // Cut current line
            let line = self.cursor.line;
            let line_text = if let Some(buffer) = self.current_buffer() {
                buffer.line(line).unwrap_or("").to_string()
            } else {
                return;
            };
            self.clipboard.copy_line(&line_text);
            if let Some(buffer) = self.current_buffer_mut() {
                buffer.delete_line(line);
                if line >= buffer.line_count() {
                    self.cursor.line = buffer.line_count().saturating_sub(1);
                }
            }
        }
    }

    fn copy_selection(&mut self) {
        let lines = self.current_lines();
        if self.selection.active && !self.selection.is_empty() {
            let text = self.selection.get_selected_text(&lines);
            self.clipboard.copy(&text, ClipKind::Character);
            self.message = Some(format!("Copied {} chars", text.len()));
        }
    }

    fn copy_line(&mut self) {
        let lines = self.current_lines();
        if let Some(line) = lines.get(self.cursor.line) {
            self.clipboard.copy_line(line);
            self.message = Some("Line copied".to_string());
        }
    }

    fn copy_function(&mut self) {
        if let Some(buffer) = self.current_buffer() {
            if let Some((start, end)) = buffer.find_function_boundaries(self.cursor.line) {
                let lines = buffer.lines();
                let text: Vec<&str> = lines[start..=end].iter().map(|l| l.as_str()).collect();
                let text = text.join("\n");
                self.clipboard.copy_function(&text);
                self.message = Some("Function copied".to_string());
            }
        }
    }

    fn copy_class(&mut self) {
        if let Some(buffer) = self.current_buffer() {
            if let Some((start, end)) = buffer.find_class_boundaries(self.cursor.line) {
                let lines = buffer.lines();
                let text: Vec<&str> = lines[start..=end].iter().map(|l| l.as_str()).collect();
                let text = text.join("\n");
                self.clipboard.copy_class(&text);
                self.message = Some("Class copied".to_string());
            }
        }
    }

    fn copy_full_file(&mut self) {
        if let Some(buffer) = self.current_buffer() {
            let text = buffer.text();
            self.clipboard.copy_file(&text);
            self.message = Some("Full file copied".to_string());
        }
    }

    fn paste(&mut self) {
        if let Some(text) = self.clipboard.paste_latest() {
            let text = text.to_string();
            let pos = self.cursor.position();
            if let Some(buffer) = self.current_buffer_mut() {
                buffer.insert_str(pos, &text);
            }
            // Move cursor to end of pasted text
            let split_lines: Vec<&str> = text.split('\n').collect();
            if split_lines.len() == 1 {
                self.cursor.col += text.len();
            } else {
                self.cursor.line += split_lines.len() - 1;
                self.cursor.col = split_lines.last().map_or(0, |l| l.len());
            }
            self.cursor.preferred_col = None;
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            // Ctrl+Q: Quit
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                self.running = false;
            }
            // Ctrl+O: Save
            (KeyModifiers::CONTROL, KeyCode::Char('o')) => {
                self.save_file();
            }
            // Ctrl+S: Save (alternative)
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                self.save_file();
            }
            // Ctrl+G: Go to line
            (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
                self.prev_mode = self.mode;
                self.mode = Mode::GoToLine;
                self.command_buffer.clear();
                self.message = Some("Go to line: ".to_string());
            }
            // Ctrl+K: Cut
            (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                self.cut_selection_or_line();
            }
            // Ctrl+U: Paste
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                self.paste();
            }
            // Ctrl+Shift+C: Copy
            (KeyModifiers::CONTROL | KeyModifiers::SHIFT, KeyCode::Char('c')) => {
                self.copy_selection();
            }
            // Ctrl+Shift+V: Clipboard history
            (KeyModifiers::CONTROL | KeyModifiers::SHIFT, KeyCode::Char('v')) => {
                self.mode = Mode::ClipboardHistory;
            }
            // Ctrl+A: Select all
            (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                let total = self.current_buffer().map_or(0, |b| b.line_count());
                self.selection.select_all(total);
            }
            // Ctrl+Z: Undo
            (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
                if let Some(buffer) = self.current_buffer_mut() {
                    if let Some(pos) = buffer.undo() {
                        self.cursor.set_position(pos);
                    }
                }
            }
            // Ctrl+Y: Redo
            (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
                if let Some(buffer) = self.current_buffer_mut() {
                    if let Some(pos) = buffer.redo() {
                        self.cursor.set_position(pos);
                    }
                }
            }
            // Ctrl+W: Search
            (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
                self.mode = Mode::Search;
                self.search_buffer.clear();
                self.message = Some("Search: ".to_string());
            }
            // Ctrl+R: Replace
            (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                self.mode = Mode::Replace;
                self.search_buffer.clear();
                self.replace_buffer.clear();
                self.message = Some("Search: ".to_string());
            }
            // Ctrl+T: Toggle explorer
            (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
                self.explorer.toggle_visibility();
                if self.explorer.is_visible() {
                    self.mode = Mode::Explorer;
                }
            }
            // Ctrl+P: Command palette
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
                self.message = Some("Command: ".to_string());
            }
            // Ctrl+D: Duplicate line
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                let line = self.cursor.line;
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.duplicate_line(line);
                    let line_count = buffer.lines().len();
                    self.cursor.move_down(&self.current_lines());
                    let _ = line_count; // buffer still valid
                }
            }
            // Ctrl+/: Toggle comment
            (KeyModifiers::CONTROL, KeyCode::Char('/')) => {
                let line = self.cursor.line;
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.comment_toggle(line);
                }
            }
            // Shift+Tab: Dedent
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                let line = self.cursor.line;
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.dedent_line(line, 4);
                }
            }
            // Alt+Up: Move line up
            (KeyModifiers::ALT, KeyCode::Up) => {
                let line = self.cursor.line;
                if let Some(buffer) = self.current_buffer_mut() {
                    if let Some(new_line) = buffer.move_line_up(line) {
                        self.cursor.line = new_line;
                    }
                }
            }
            // Alt+Down: Move line down
            (KeyModifiers::ALT, KeyCode::Down) => {
                let line = self.cursor.line;
                if let Some(buffer) = self.current_buffer_mut() {
                    if let Some(new_line) = buffer.move_line_down(line) {
                        self.cursor.line = new_line;
                    }
                }
            }
            // Alt+Right: Next tab
            (KeyModifiers::ALT, KeyCode::Right) => {
                self.tab_manager.next_tab();
                self.cursor = Cursor::new();
            }
            // Alt+Left: Previous tab
            (KeyModifiers::ALT, KeyCode::Left) => {
                self.tab_manager.prev_tab();
                self.cursor = Cursor::new();
            }
            // F1: Help
            (_, KeyCode::F(1)) => {
                self.mode = Mode::Help;
            }

            // Movement keys
            (_, KeyCode::Up) => {
                let lines = self.current_lines();
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    if !self.selection.active {
                        self.selection.start_selection(self.cursor.position(), SelectionMode::Character);
                    }
                    self.cursor.move_up(&lines);
                    self.selection.update_cursor(self.cursor.position());
                } else {
                    self.selection.clear();
                    self.cursor.move_up(&lines);
                }
            }
            (_, KeyCode::Down) => {
                let lines = self.current_lines();
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    if !self.selection.active {
                        self.selection.start_selection(self.cursor.position(), SelectionMode::Character);
                    }
                    self.cursor.move_down(&lines);
                    self.selection.update_cursor(self.cursor.position());
                } else {
                    self.selection.clear();
                    self.cursor.move_down(&lines);
                }
            }
            (_, KeyCode::Left) => {
                let lines = self.current_lines();
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    if !self.selection.active {
                        self.selection.start_selection(self.cursor.position(), SelectionMode::Character);
                    }
                    self.cursor.move_left(&lines);
                    self.selection.update_cursor(self.cursor.position());
                } else {
                    if self.selection.active {
                        self.cursor.set_position(self.selection.start());
                    }
                    self.selection.clear();
                    self.cursor.move_left(&lines);
                }
            }
            (_, KeyCode::Right) => {
                let lines = self.current_lines();
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    if !self.selection.active {
                        self.selection.start_selection(self.cursor.position(), SelectionMode::Character);
                    }
                    self.cursor.move_right(&lines);
                    self.selection.update_cursor(self.cursor.position());
                } else {
                    if self.selection.active {
                        self.cursor.set_position(self.selection.end());
                    }
                    self.selection.clear();
                    self.cursor.move_right(&lines);
                }
            }
            (_, KeyCode::Home) => {
                self.cursor.move_to_line_start();
            }
            (_, KeyCode::End) => {
                let lines = self.current_lines();
                self.cursor.move_to_line_end(&lines);
            }
            (_, KeyCode::PageUp) => {
                let lines = self.current_lines();
                self.cursor.move_page_up(&lines, self.view.visible_height);
            }
            (_, KeyCode::PageDown) => {
                let lines = self.current_lines();
                self.cursor.move_page_down(&lines, self.view.visible_height);
            }

            // Enter insert mode
            (_, KeyCode::Enter) => {
                self.mode = Mode::Insert;
                self.message = None;
            }
            // Any printable character enters insert mode
            (_, KeyCode::Char(c)) => {
                self.mode = Mode::Insert;
                // Insert the character
                let pos = self.cursor.position();
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.insert_char(pos, c);
                }
                self.cursor.col += 1;
                self.cursor.preferred_col = None;
            }
            // Delete key
            (_, KeyCode::Delete) => {
                let pos = self.cursor.position();
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.delete_char(pos);
                }
            }
            // Backspace
            (_, KeyCode::Backspace) => {
                let pos = self.cursor.position();
                if let Some(buffer) = self.current_buffer_mut() {
                    if let Some(new_pos) = buffer.backspace(pos) {
                        self.cursor.set_position(new_pos);
                    }
                }
            }
            // Tab
            (_, KeyCode::Tab) => {
                let line = self.cursor.line;
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.indent_line(line, 4, true);
                }
                self.cursor.col += 4;
            }
            _ => {}
        }
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            // Escape: back to normal mode
            (_, KeyCode::Esc) => {
                self.mode = Mode::Normal;
                self.message = None;
            }
            // Ctrl+O: Save without leaving insert
            (KeyModifiers::CONTROL, KeyCode::Char('o')) => {
                self.save_file();
            }
            // Ctrl+S: Save
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                self.save_file();
            }
            // Ctrl+Z: Undo
            (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
                if let Some(buffer) = self.current_buffer_mut() {
                    if let Some(pos) = buffer.undo() {
                        self.cursor.set_position(pos);
                    }
                }
            }
            // Enter: new line
            (_, KeyCode::Enter) => {
                let pos = self.cursor.position();
                let new_pos = if let Some(buffer) = self.current_buffer_mut() {
                    buffer.insert_newline(pos)
                } else {
                    return;
                };
                self.cursor.set_position(new_pos);

                // Auto-indent
                if self.should_auto_indent() {
                    let prev_line_idx = self.cursor.line.saturating_sub(1);
                    let prev_line = self.current_buffer()
                        .and_then(|b| b.line(prev_line_idx))
                        .unwrap_or("")
                        .to_string();
                    let indent: String = prev_line.chars().take_while(|c| *c == ' ' || *c == '\t').collect();

                    // Extra indent after {
                    let extra = if prev_line.trim_end().ends_with('{')
                        || prev_line.trim_end().ends_with(':')
                        || prev_line.trim_end().ends_with('(')
                    {
                        "    "
                    } else {
                        ""
                    };

                    let full_indent = format!("{}{}", indent, extra);
                    if !full_indent.is_empty() {
                        let insert_pos = BufferPosition::new(self.cursor.line, 0);
                        if let Some(buffer) = self.current_buffer_mut() {
                            buffer.insert_str(insert_pos, &full_indent);
                        }
                        self.cursor.col = full_indent.len();
                    }
                }
            }
            // Tab
            (_, KeyCode::Tab) => {
                let settings = Settings::new();
                let pos = self.cursor.position();
                if settings.use_spaces {
                    let spaces = " ".repeat(settings.tab_size);
                    if let Some(buffer) = self.current_buffer_mut() {
                        buffer.insert_str(pos, &spaces);
                    }
                    self.cursor.col += settings.tab_size;
                } else {
                    if let Some(buffer) = self.current_buffer_mut() {
                        buffer.insert_char(pos, '\t');
                    }
                    self.cursor.col += 1;
                }
            }
            // Shift+Tab: Dedent
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                let line = self.cursor.line;
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.dedent_line(line, 4);
                }
            }
            // Backspace
            (_, KeyCode::Backspace) => {
                let pos = self.cursor.position();
                if let Some(buffer) = self.current_buffer_mut() {
                    if let Some(new_pos) = buffer.backspace(pos) {
                        self.cursor.set_position(new_pos);
                    }
                }
            }
            // Delete
            (_, KeyCode::Delete) => {
                let pos = self.cursor.position();
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.delete_char(pos);
                }
            }
            // Movement keys (stay in insert mode)
            (_, KeyCode::Up) => {
                let lines = self.current_lines();
                self.cursor.move_up(&lines);
            }
            (_, KeyCode::Down) => {
                let lines = self.current_lines();
                self.cursor.move_down(&lines);
            }
            (_, KeyCode::Left) => {
                let lines = self.current_lines();
                self.cursor.move_left(&lines);
            }
            (_, KeyCode::Right) => {
                let lines = self.current_lines();
                self.cursor.move_right(&lines);
            }
            (_, KeyCode::Home) => {
                self.cursor.move_to_line_start();
            }
            (_, KeyCode::End) => {
                let lines = self.current_lines();
                self.cursor.move_to_line_end(&lines);
            }
            (_, KeyCode::PageUp) => {
                let lines = self.current_lines();
                self.cursor.move_page_up(&lines, self.view.visible_height);
            }
            (_, KeyCode::PageDown) => {
                let lines = self.current_lines();
                self.cursor.move_page_down(&lines, self.view.visible_height);
            }
            // Normal character
            (_, KeyCode::Char(c)) => {
                let pos = self.cursor.position();
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.insert_char(pos, c);
                }
                self.cursor.col += 1;
                self.cursor.preferred_col = None;
            }
            _ => {}
        }
    }

    fn handle_search_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.message = None;
            }
            KeyCode::Enter => {
                let lines = self.current_lines();
                if let Some(result) = self.search_engine.search_next(
                    &self.search_buffer,
                    &lines,
                    BufferPosition::new(self.cursor.line, self.cursor.col + 1),
                ) {
                    self.cursor.line = result.line;
                    self.cursor.col = result.col;
                    self.selection.start_selection(
                        BufferPosition::new(result.line, result.col),
                        SelectionMode::Character,
                    );
                    self.selection.update_cursor(BufferPosition::new(
                        result.line,
                        result.col + result.text.len(),
                    ));
                    self.view.center_on_line(result.line, lines.len());
                } else {
                    self.message = Some("No match found".to_string());
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                self.search_buffer.pop();
                self.message = Some(format!("Search: {}", self.search_buffer));
            }
            KeyCode::Char(c) => {
                self.search_buffer.push(c);
                // Live search: jump to first match
                let lines = self.current_lines();
                if let Some(result) = self.search_engine.search_next(
                    &self.search_buffer,
                    &lines,
                    BufferPosition::new(self.cursor.line, self.cursor.col),
                ) {
                    self.cursor.line = result.line;
                    self.cursor.col = result.col;
                    self.view.center_on_line(result.line, lines.len());
                }
                self.message = Some(format!("Search: {}", self.search_buffer));
            }
            _ => {}
        }
    }

    fn handle_replace_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.message = None;
            }
            KeyCode::Enter => {
                // If we have a search term but no replace term, search first
                if !self.search_buffer.is_empty() {
                    let lines = self.current_lines();
                    if let Some(_result) = self.search_engine.search_next(
                        &self.search_buffer,
                        &lines,
                        BufferPosition::zero(),
                    ) {
                        self.message = Some("Replace with: ".to_string());
                        // Second phase: enter replacement text
                    }
                }
            }
            KeyCode::Backspace => {
                self.search_buffer.pop();
                self.message = Some(format!("Search: {}", self.search_buffer));
            }
            KeyCode::Char(c) => {
                self.search_buffer.push(c);
                self.message = Some(format!("Search: {}", self.search_buffer));
            }
            _ => {}
        }
    }

    fn handle_command_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.message = None;
            }
            KeyCode::Enter => {
                self.execute_command(&self.command_buffer.clone());
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                self.message = Some(format!("Command: {}", self.command_buffer));
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
                self.message = Some(format!("Command: {}", self.command_buffer));
            }
            _ => {}
        }
    }

    fn handle_goto_line_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.message = None;
            }
            KeyCode::Enter => {
                if let Ok(line_num) = self.command_buffer.parse::<usize>() {
                    let lines = self.current_lines();
                    let total = lines.len();
                    if line_num > 0 && line_num <= total {
                        self.cursor.goto_line(line_num - 1, total, &lines);
                        self.view.center_on_line(line_num - 1, total);
                        self.message = Some(format!("Went to line {}", line_num));
                    } else {
                        self.message = Some(format!("Line {} out of range (1-{})", line_num, total));
                    }
                } else {
                    self.message = Some("Invalid line number".to_string());
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                self.message = Some(format!("Go to line: {}", self.command_buffer));
            }
            KeyCode::Char(c) => {
                if c.is_ascii_digit() {
                    self.command_buffer.push(c);
                }
                self.message = Some(format!("Go to line: {}", self.command_buffer));
            }
            _ => {}
        }
    }

    fn handle_explorer_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.explorer.hide();
                self.mode = Mode::Normal;
                self.message = None;
            }
            KeyCode::Up => {
                self.explorer.select_up();
            }
            KeyCode::Down => {
                self.explorer.select_down();
            }
            KeyCode::Enter => {
                if let Some(path) = self.explorer.toggle_expand(self.explorer.selected_index()) {
                    if !path.is_dir() {
                        self.open_file(path);
                        self.explorer.hide();
                        self.mode = Mode::Normal;
                    }
                }
            }
            KeyCode::Char('n') => {
                // New file - simplified
                self.message = Some("New file name: ".to_string());
                self.mode = Mode::Command;
                self.command_buffer = "newfile ".to_string();
            }
            KeyCode::Char('d') => {
                // Delete file/directory
                if let Some(entry) = self.explorer.selected() {
                    if let Err(e) = self.explorer.delete(&entry.path) {
                        self.message = Some(format!("Error: {}", e));
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_help_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
                self.message = None;
            }
            _ => {}
        }
    }

    fn handle_clipboard_history_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.message = None;
            }
            KeyCode::Up => {
                // Navigate up in clipboard history
            }
            KeyCode::Down => {
                // Navigate down in clipboard history
            }
            KeyCode::Enter => {
                // Paste selected entry
                if let Some(text) = self.clipboard.paste_by_index(0) {
                    let text = text.to_string();
                    let pos = self.cursor.position();
                    if let Some(buffer) = self.current_buffer_mut() {
                        buffer.insert_str(pos, &text);
                    }
                }
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn execute_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "w" | "write" | "save" => {
                self.save_file();
            }
            "q" | "quit" | "exit" => {
                self.running = false;
            }
            "wq" | "x" => {
                self.save_file();
                self.running = false;
            }
            "theme" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "dark" => self.message = Some("Theme: Dark (restart to apply)".to_string()),
                        "light" => self.message = Some("Theme: Light (restart to apply)".to_string()),
                        _ => self.message = Some("Unknown theme. Use: dark, light".to_string()),
                    }
                } else {
                    self.message = Some("Usage: theme <dark|light>".to_string());
                }
            }
            "git" => {
                if !self.git.is_available() {
                    self.message = Some("Not a git repository".to_string());
                    return;
                }
                match parts.get(1).copied() {
                    Some("status") => {
                        let status = self.git.format_status();
                        self.message = Some(format!("Git Status:\n{}", status));
                    }
                    Some("diff") => {
                        let file = parts.get(2).copied().unwrap_or("");
                        let diff = self.git.format_diff(file);
                        self.message = Some(format!("Git Diff:\n{}", diff));
                    }
                    Some("commit") => {
                        let msg = if parts.len() > 2 {
                            parts[2..].join(" ")
                        } else {
                            "Update".to_string()
                        };
                        match self.git.commit(&msg) {
                            Ok(id) => self.message = Some(format!("Committed: {}", id)),
                            Err(e) => self.message = Some(format!("Error: {}", e)),
                        }
                    }
                    Some("branch") => {
                        match self.git.current_branch() {
                            Ok(b) => self.message = Some(format!("Branch: {}", b)),
                            Err(e) => self.message = Some(format!("Error: {}", e)),
                        }
                    }
                    Some("log") => {
                        match self.git.log(10) {
                            Ok(entries) => {
                                let mut output = String::new();
                                for (id, msg, author) in &entries {
                                    output.push_str(&format!("{} {} ({})\n", id, msg, author));
                                }
                                self.message = Some(output);
                            }
                            Err(e) => self.message = Some(format!("Error: {}", e)),
                        }
                    }
                    _ => {
                        self.message = Some("Git commands: status, diff, commit, branch, log".to_string());
                    }
                }
            }
            "find" | "search" => {
                if parts.len() > 1 {
                    let query = parts[1..].join(" ");
                    if let Some(root) = self.explorer.root() {
                        let results = self.global_search.search_in_project(&query, root, None);
                        let mut output = format!("Found {} results:\n", results.len());
                        for result in results.iter().take(20) {
                            if let Some(path) = &result.path {
                                output.push_str(&format!("  {}:{} - {}\n",
                                    path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
                                    result.line + 1,
                                    result.text.trim()
                                ));
                            }
                        }
                        self.message = Some(output);
                    } else {
                        self.message = Some("No project root found".to_string());
                    }
                }
            }
            "ai" => {
                self.message = Some("AI Assistant - Coming Soon".to_string());
            }
            "about" => {
                self.mode = Mode::About;
            }
            "newfile" => {
                if parts.len() > 1 {
                    let name = parts[1..].join(" ");
                    if let Some(root) = self.explorer.root() {
                        let path = root.join(&name);
                        match self.explorer.create_file(&path) {
                            Ok(()) => {
                                self.open_file(path);
                                self.message = Some(format!("Created: {}", name));
                            }
                            Err(e) => self.message = Some(format!("Error: {}", e)),
                        }
                    }
                }
            }
            _ => {
                self.message = Some(format!("Unknown command: {}", parts[0]));
            }
        }
    }

    fn should_auto_indent(&self) -> bool {
        true // Always auto-indent in V1
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
            Mode::Search => self.handle_search_mode(key),
            Mode::Replace => self.handle_replace_mode(key),
            Mode::Command => self.handle_command_mode(key),
            Mode::Explorer => self.handle_explorer_mode(key),
            Mode::GoToLine => self.handle_goto_line_mode(key),
            Mode::Help => self.handle_help_mode(key),
            Mode::ClipboardHistory => self.handle_clipboard_history_mode(key),
            Mode::SaveAs | Mode::OpenFile => self.handle_command_mode(key),
            Mode::GitStatus => self.handle_help_mode(key),
            Mode::About => self.handle_help_mode(key),
        }
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(button) => {
                let col = mouse.column as usize;
                let row = mouse.row as usize;

                // Adjust for tab bar (row 0) and status bars
                let editor_row = row.saturating_sub(1);
                let line_idx = self.view.scroll_y + editor_row;

                match button {
                    crossterm::event::MouseButton::Left => {
                        if line_idx < self.current_buffer().map_or(0, |b| b.line_count()) {
                            let content_col = col.saturating_sub(self.view.content_start_col()) + self.view.scroll_x;
                            self.cursor.line = line_idx;
                            self.cursor.col = content_col;
                            self.cursor.clamp_to_line(&self.current_lines());
                            self.selection.start_selection(self.cursor.position(), SelectionMode::Character);
                        }
                    }
                    crossterm::event::MouseButton::Right => {
                        // Right click: copy selection
                        self.copy_selection();
                    }
                    _ => {}
                }
            }
            MouseEventKind::Drag(_) => {
                let col = mouse.column as usize;
                let row = mouse.row as usize;

                // Adjust for tab bar
                let editor_row = row.saturating_sub(1);
                let line_idx = self.view.scroll_y + editor_row;

                if line_idx < self.current_buffer().map_or(0, |b| b.line_count()) {
                    let content_col = col.saturating_sub(self.view.content_start_col()) + self.view.scroll_x;
                    self.cursor.line = line_idx;
                    self.cursor.col = content_col;
                    self.cursor.clamp_to_line(&self.current_lines());
                    self.selection.update_cursor(self.cursor.position());
                }
            }
            MouseEventKind::ScrollUp => {
                self.view.scroll_up(3);
            }
            MouseEventKind::ScrollDown => {
                let total = self.current_buffer().map_or(0, |b| b.line_count());
                self.view.scroll_down(3, total);
            }
            _ => {}
        }
    }
}

fn print_about() {
    println!();
    println!("  +======================================+");
    println!("  |                                      |");
    println!("  |       Vertil Nano Pro v{}        |", VERSION);
    println!("  |                                      |");
    println!("  |    Modern Terminal Code Editor       |");
    println!("  |                                      |");
    println!("  |    Created by {}          |", AUTHOR);
    println!("  |                                      |");
    println!("  +======================================+");
    println!();
    println!("  Name:       Vertil Nano Pro");
    println!("  Version:    {}", VERSION);
    println!("  Author:     {}", AUTHOR);
    println!("  License:    {}", LICENSE);
    println!("  Repository: {}", REPOSITORY);
    println!();
}

fn run_editor(files: Vec<PathBuf>) -> io::Result<()> {
    let settings = Settings::load_or_create();
    let mut renderer = Renderer::new(&settings);
    let mut app = App::new(&settings);

    renderer.init_terminal()?;

    // Open files
    if files.is_empty() {
        let _ = app.tab_manager.open_tab(None);
        app.message = Some("Vertil Nano Pro - Press Enter to start editing, Ctrl+Q to quit".to_string());
    } else {
        for file in &files {
            app.open_file(file.clone());
        }
    }

    // Show splash screen briefly
    renderer.render_splash()?;
    let _ = event::read(); // Wait for any key

    // Main event loop
    while app.running {
        // Render
        let message = app.message.as_deref();
        if let Some(tab) = app.tab_manager.active() {
            let lines = tab.buffer.lines().to_vec();
            renderer.render(
                &app.tab_manager,
                &mut app.view,
                &app.cursor,
                &app.selection,
                message,
            )?;
        } else {
            renderer.render_empty()?;
        }

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    app.handle_key_event(key);
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse_event(mouse);
                }
                Event::Resize(width, height) => {
                    app.view.resize(width as usize, height as usize);
                }
                Event::Paste(text) => {
                    let pos = app.cursor.position();
                    if let Some(buffer) = app.current_buffer_mut() {
                        buffer.insert_str(pos, &text);
                    }
                }
                _ => {}
            }
        }
    }

    renderer.restore_terminal()?;
    Ok(())
}

fn main() {
    // Initialize logger
    env_logger::init();

    let cli = clap::Command::new("Vertil Nano Pro")
        .version(VERSION)
        .author(AUTHOR)
        .about("Modern terminal code editor — simple like Nano, powerful for 2026")
        .arg(
            clap::Arg::new("files")
                .help("Files to open")
                .num_args(0..)
                .index(1),
        )
        .arg(
            clap::Arg::new("about")
                .long("about")
                .help("Show information about Vertil Nano Pro")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("version")
                .short('v')
                .long("version")
                .help("Show version")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("theme")
                .long("theme")
                .help("Set theme (dark/light)")
                .value_name("THEME"),
        );

    let matches = cli.get_matches();

    if matches.get_flag("about") {
        print_about();
        return;
    }

    if matches.get_flag("version") {
        println!("Vertil Nano Pro v{}", VERSION);
        return;
    }

    let files: Vec<PathBuf> = matches
        .get_many::<String>("files")
        .unwrap_or_default()
        .map(PathBuf::from)
        .collect();

    if let Err(e) = run_editor(files) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
