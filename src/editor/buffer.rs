use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct BufferChange {
    pub kind: ChangeKind,
    pub range: BufferRange,
    pub text: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChangeKind {
    Insert,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferPosition {
    pub line: usize,
    pub col: usize,
}

impl BufferPosition {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    pub fn zero() -> Self {
        Self { line: 0, col: 0 }
    }

    pub fn min(self, other: Self) -> Self {
        if self.line < other.line || (self.line == other.line && self.col < other.col) {
            self
        } else {
            other
        }
    }

    pub fn max(self, other: Self) -> Self {
        if self.line > other.line || (self.line == other.line && self.col > other.col) {
            self
        } else {
            other
        }
    }
}

impl std::cmp::PartialOrd for BufferPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for BufferPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.line.cmp(&other.line) {
            std::cmp::Ordering::Equal => self.col.cmp(&other.col),
            ord => ord,
        }
    }
}

impl Eq for BufferPosition {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferRange {
    pub start: BufferPosition,
    pub end: BufferPosition,
}

impl BufferRange {
    pub fn new(start: BufferPosition, end: BufferPosition) -> Self {
        if start <= end {
            Self { start, end }
        } else {
            Self { start: end, end: start }
        }
    }

    pub fn single(pos: BufferPosition) -> Self {
        Self { start: pos, end: pos }
    }

    pub fn line(line: usize) -> Self {
        Self {
            start: BufferPosition::new(line, 0),
            end: BufferPosition::new(line, usize::MAX),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn contains(&self, pos: BufferPosition) -> bool {
        pos >= self.start && pos <= self.end
    }
}

pub struct Buffer {
    lines: Vec<String>,
    path: Option<PathBuf>,
    dirty: bool,
    undo_stack: Vec<BufferChange>,
    redo_stack: Vec<BufferChange>,
    last_save_time: Option<Instant>,
    file_encoding: String,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            path: None,
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_save_time: None,
            file_encoding: "UTF-8".to_string(),
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;

        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }

        // If the file ends with a newline, keep the trailing empty line representation
        if content.ends_with('\n') && !lines.last().map_or(false, |l| l.is_empty()) {
            lines.push(String::new());
        }

        Ok(Self {
            lines,
            path: Some(path.to_path_buf()),
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_save_time: Some(Instant::now()),
            file_encoding: "UTF-8".to_string(),
        })
    }

    pub fn from_content(content: &str, path: Option<PathBuf>) -> Self {
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        if content.ends_with('\n') && !lines.last().map_or(false, |l| l.is_empty()) {
            lines.push(String::new());
        }

        Self {
            lines,
            path,
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_save_time: None,
            file_encoding: "UTF-8".to_string(),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    pub fn filename(&self) -> &str {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[New File]")
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }

    pub fn line_mut(&mut self, line: usize) -> Option<&mut String> {
        self.lines.get_mut(line)
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line_len(&self, line: usize) -> usize {
        self.lines.get(line).map_or(0, |l| l.len())
    }

    pub fn char_count(&self) -> usize {
        self.lines.iter().map(|l| l.len()).sum::<usize>() + self.lines.len().saturating_sub(1)
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty() && !self.dirty
    }

    pub fn insert_char(&mut self, pos: BufferPosition, ch: char) {
        if pos.line >= self.lines.len() {
            return;
        }
        let line = &mut self.lines[pos.line];
        let col = pos.col.min(line.len());
        line.insert(col, ch);
        self.dirty = true;
        self.redo_stack.clear();
    }

    pub fn insert_str(&mut self, pos: BufferPosition, text: &str) {
        if pos.line >= self.lines.len() || text.is_empty() {
            return;
        }

        let col = pos.col.min(self.lines[pos.line].len());
        let lines_to_insert: Vec<&str> = text.split('\n').collect();

        if lines_to_insert.len() == 1 {
            self.lines[pos.line].insert_str(col, text);
        } else {
            let original_line = self.lines[pos.line].clone();
            let before = original_line[..col].to_string();
            let after = original_line[col..].to_string();

            self.lines[pos.line] = format!("{}{}", before, lines_to_insert[0]);

            let mut new_lines = Vec::new();
            for (i, &segment) in lines_to_insert.iter().enumerate() {
                if i == 0 {
                    continue;
                }
                if i == lines_to_insert.len() - 1 {
                    new_lines.push(format!("{}{}", segment, after));
                } else {
                    new_lines.push(segment.to_string());
                }
            }

            for (offset, line) in new_lines.into_iter().enumerate() {
                self.lines.insert(pos.line + 1 + offset, line);
            }
        }

        self.dirty = true;
        self.redo_stack.clear();
    }

    pub fn delete_char(&mut self, pos: BufferPosition) -> bool {
        if pos.line >= self.lines.len() {
            return false;
        }

        let line_len = self.lines[pos.line].len();

        if pos.col < line_len {
            self.lines[pos.line].remove(pos.col);
            self.dirty = true;
            self.redo_stack.clear();
            return true;
        } else if pos.col == line_len && pos.line + 1 < self.lines.len() {
            let next_line = self.lines.remove(pos.line + 1);
            self.lines[pos.line].push_str(&next_line);
            self.dirty = true;
            self.redo_stack.clear();
            return true;
        }

        false
    }

    pub fn backspace(&mut self, pos: BufferPosition) -> Option<BufferPosition> {
        if pos.line >= self.lines.len() {
            return None;
        }

        if pos.col > 0 {
            self.lines[pos.line].remove(pos.col - 1);
            self.dirty = true;
            self.redo_stack.clear();
            return Some(BufferPosition::new(pos.line, pos.col - 1));
        } else if pos.line > 0 {
            let prev_line_len = self.lines[pos.line - 1].len();
            let current_line = self.lines.remove(pos.line);
            self.lines[pos.line - 1].push_str(&current_line);
            self.dirty = true;
            self.redo_stack.clear();
            return Some(BufferPosition::new(pos.line - 1, prev_line_len));
        }

        None
    }

    pub fn insert_newline(&mut self, pos: BufferPosition) -> BufferPosition {
        if pos.line >= self.lines.len() {
            return pos;
        }

        let col = pos.col.min(self.lines[pos.line].len());
        let current_line = self.lines[pos.line].clone();
        let before = current_line[..col].to_string();
        let after = current_line[col..].to_string();

        self.lines[pos.line] = before;
        self.lines.insert(pos.line + 1, after);

        self.dirty = true;
        self.redo_stack.clear();

        BufferPosition::new(pos.line + 1, 0)
    }

    pub fn delete_range(&mut self, range: BufferRange) {
        let start = range.start;
        let end = range.end;

        if start.line >= self.lines.len() || end.line >= self.lines.len() {
            return;
        }

        if start.line == end.line {
            let line = &mut self.lines[start.line];
            let s = start.col.min(line.len());
            let e = end.col.min(line.len());
            if s < e {
                line.drain(s..e);
            }
        } else {
            let first_part = self.lines[start.line][..start.col.min(self.lines[start.line].len())].to_string();
            let last_part = self.lines[end.line][end.col.min(self.lines[end.line].len())..].to_string();

            self.lines[start.line] = format!("{}{}", first_part, last_part);

            let delete_count = end.line - start.line;
            for _ in 0..delete_count {
                if start.line + 1 < self.lines.len() {
                    self.lines.remove(start.line + 1);
                }
            }
        }

        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        self.dirty = true;
        self.redo_stack.clear();
    }

    pub fn replace_range(&mut self, range: BufferRange, text: &str) {
        self.delete_range(range);
        self.insert_str(range.start, text);
    }

    pub fn delete_line(&mut self, line: usize) {
        if line >= self.lines.len() {
            return;
        }
        self.lines.remove(line);
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.dirty = true;
        self.redo_stack.clear();
    }

    pub fn insert_line(&mut self, after_line: usize, content: String) {
        let insert_pos = (after_line + 1).min(self.lines.len());
        self.lines.insert(insert_pos, content);
        self.dirty = true;
        self.redo_stack.clear();
    }

    pub fn indent_line(&mut self, line: usize, tab_size: usize, use_spaces: bool) {
        if line >= self.lines.len() {
            return;
        }
        if use_spaces {
            self.lines[line].insert_str(0, &" ".repeat(tab_size));
        } else {
            self.lines[line].insert(0, '\t');
        }
        self.dirty = true;
    }

    pub fn dedent_line(&mut self, line: usize, tab_size: usize) {
        if line >= self.lines.len() {
            return;
        }
        let line_str = &mut self.lines[line];
        if line_str.starts_with('\t') {
            line_str.remove(0);
        } else {
            let spaces = line_str.chars().take_while(|c| *c == ' ').count();
            let remove_count = spaces.min(tab_size);
            line_str.drain(..remove_count);
        }
        self.dirty = true;
    }

    pub fn save(&mut self) -> Result<(), String> {
        if let Some(path) = &self.path {
            self.save_to(path)
        } else {
            Err("No file path set. Use save_as.".to_string())
        }
    }

    pub fn save_to(&mut self, path: &Path) -> Result<(), String> {
        let content = self.text();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        fs::write(path, content)
            .map_err(|e| format!("Failed to write file '{}': {}", path.display(), e))?;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        self.last_save_time = Some(Instant::now());
        Ok(())
    }

    pub fn last_save_time(&self) -> Option<Instant> {
        self.last_save_time
    }

    pub fn language_id(&self) -> &str {
        self.path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .map(|ext| match ext {
                "py" | "pyw" => "python",
                "js" | "mjs" | "cjs" => "javascript",
                "ts" => "typescript",
                "tsx" => "typescriptreact",
                "jsx" => "javascriptreact",
                "go" => "go",
                "rs" => "rust",
                "c" | "h" => "c",
                "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
                "java" => "java",
                "rb" => "ruby",
                "sh" | "bash" => "shell",
                "json" => "json",
                "md" => "markdown",
                "toml" => "toml",
                "yaml" | "yml" => "yaml",
                "html" => "html",
                "css" => "css",
                _ => "plaintext",
            })
            .unwrap_or("plaintext")
    }

    pub fn push_undo(&mut self, change: BufferChange) {
        self.undo_stack.push(change);
        if self.undo_stack.len() > 1000 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> Option<BufferPosition> {
        let change = self.undo_stack.pop()?;
        match change.kind {
            ChangeKind::Insert => {
                // Undo insert = delete
                self.delete_range(change.range);
                Some(change.range.start)
            }
            ChangeKind::Delete => {
                // Undo delete = insert
                self.insert_str(change.range.start, &change.text);
                Some(change.range.start)
            }
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn redo(&mut self) -> Option<BufferPosition> {
        let change = self.redo_stack.pop()?;
        match change.kind {
            ChangeKind::Insert => {
                self.insert_str(change.range.start, &change.text);
                Some(BufferPosition::new(
                    change.range.end.line,
                    change.range.end.col,
                ))
            }
            ChangeKind::Delete => {
                self.delete_range(change.range);
                Some(change.range.start)
            }
        }
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn find_function_boundaries(&self, cursor_line: usize) -> Option<(usize, usize)> {
        let lang = self.language_id();
        let mut start_line = None;
        let mut end_line = None;

        let patterns: &[&str] = match lang {
            "python" => &["def ", "async def ", "class "],
            "javascript" | "typescript" | "typescriptreact" | "javascriptreact" => {
                &["function ", "async function ", "const ", "let ", "var ", "class "]
            }
            "go" => &["func ", "type ", "interface "],
            "rust" => &["fn ", "pub fn ", "struct ", "enum ", "impl ", "trait "],
            "c" | "cpp" => &["void ", "int ", "char ", "float ", "double ", "class ", "struct "],
            _ => &["function ", "def ", "fn "],
        };

        // Search backward for function/class start
        for line_num in (0..=cursor_line).rev() {
            let line = self.lines.get(line_num)?;
            let trimmed = line.trim();
            for pattern in patterns {
                if trimmed.starts_with(pattern) {
                    start_line = Some(line_num);
                    break;
                }
            }
            if start_line.is_some() {
                break;
            }
        }

        let start = start_line?;

        // Search forward for function/class end
        let base_indent = self.lines.get(start).map_or(0, |l| l.len() - l.trim_start().len());
        for line_num in (start + 1)..self.lines.len() {
            let line = self.lines.get(line_num)?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let current_indent = line.len() - line.trim_start().len();

            // If we find something at same or lesser indentation that isn't empty, we've passed the block
            if current_indent <= base_indent {
                for pattern in patterns {
                    if trimmed.starts_with(pattern) {
                        end_line = Some(line_num.saturating_sub(1));
                        break;
                    }
                }
                if end_line.is_some() {
                    break;
                }
                end_line = Some(line_num.saturating_sub(1));
                break;
            }
        }

        Some((start, end_line.unwrap_or(self.lines.len().saturating_sub(1))))
    }

    pub fn find_class_boundaries(&self, cursor_line: usize) -> Option<(usize, usize)> {
        let lang = self.language_id();
        let class_patterns: &[&str] = match lang {
            "python" => &["class "],
            "javascript" | "typescript" | "typescriptreact" | "javascriptreact" => &["class "],
            "go" => &["type ", "struct "],
            "rust" => &["struct ", "enum ", "impl ", "trait "],
            "c" | "cpp" => &["class ", "struct "],
            _ => &["class "],
        };

        let mut start_line = None;
        for line_num in (0..=cursor_line).rev() {
            let line = self.lines.get(line_num)?;
            let trimmed = line.trim();
            for pattern in class_patterns {
                if trimmed.starts_with(pattern) {
                    start_line = Some(line_num);
                    break;
                }
            }
            if start_line.is_some() {
                break;
            }
        }

        let start = start_line?;
        let base_indent = self.lines.get(start).map_or(0, |l| l.len() - l.trim_start().len());

        let mut end_line = self.lines.len().saturating_sub(1);
        for line_num in (start + 1)..self.lines.len() {
            let line = self.lines.get(line_num)?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let current_indent = line.len() - line.trim_start().len();
            if current_indent <= base_indent {
                end_line = line_num.saturating_sub(1);
                break;
            }
        }

        Some((start, end_line))
    }

    pub fn duplicate_line(&mut self, line: usize) {
        if line >= self.lines.len() {
            return;
        }
        let dup = self.lines[line].clone();
        self.lines.insert(line + 1, dup);
        self.dirty = true;
        self.redo_stack.clear();
    }

    pub fn move_line_up(&mut self, line: usize) -> Option<usize> {
        if line == 0 || line >= self.lines.len() {
            return None;
        }
        self.lines.swap(line, line - 1);
        self.dirty = true;
        Some(line - 1)
    }

    pub fn move_line_down(&mut self, line: usize) -> Option<usize> {
        if line + 1 >= self.lines.len() {
            return None;
        }
        self.lines.swap(line, line + 1);
        self.dirty = true;
        Some(line + 1)
    }

    pub fn comment_toggle(&mut self, line: usize) {
        if line >= self.lines.len() {
            return;
        }
        let lang = self.language_id();
        let comment_str = match lang {
            "python" | "ruby" | "shell" => "#",
            "javascript" | "typescript" | "typescriptreact" | "javascriptreact" | "go" | "c" | "cpp" | "rust" | "java" => "//",
            "html" | "xml" => "<!--",
            "css" => "/*",
            _ => "//",
        };

        let line_content = self.lines[line].clone();
        let trimmed = line_content.trim_start();

        if trimmed.starts_with(comment_str) {
            // Uncomment
            if let Some(idx) = line_content.find(comment_str) {
                let after_comment = &line_content[idx + comment_str.len()..];
                let before_comment = &line_content[..idx];
                let new_content = format!("{}{}", before_comment, after_comment.trim_start());
                self.lines[line] = new_content;
            }
        } else {
            // Comment
            let indent = line_content.len() - line_content.trim_start().len();
            let new_content = format!("{}{} {}", &line_content[..indent], comment_str, trimmed);
            self.lines[line] = new_content;
        }
        self.dirty = true;
    }
}
