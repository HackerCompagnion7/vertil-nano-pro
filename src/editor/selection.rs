use crate::editor::buffer::BufferPosition;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionMode {
    Character,
    Line,
    Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    pub active: bool,
    pub mode: SelectionMode,
    pub anchor: BufferPosition,
    pub cursor: BufferPosition,
    pub block_anchor_col: Option<usize>,
    pub block_cursor_col: Option<usize>,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            active: false,
            mode: SelectionMode::Character,
            anchor: BufferPosition::zero(),
            cursor: BufferPosition::zero(),
            block_anchor_col: None,
            block_cursor_col: None,
        }
    }

    pub fn start(&self) -> BufferPosition {
        self.anchor.min(self.cursor)
    }

    pub fn end(&self) -> BufferPosition {
        self.anchor.max(self.cursor)
    }

    pub fn is_empty(&self) -> bool {
        match self.mode {
            SelectionMode::Character => self.anchor == self.cursor,
            SelectionMode::Line => self.anchor.line == self.cursor.line,
            SelectionMode::Block => {
                self.anchor.line == self.cursor.line
                    && self.block_anchor_col == self.block_cursor_col
            }
        }
    }

    pub fn clear(&mut self) {
        self.active = false;
        self.anchor = BufferPosition::zero();
        self.cursor = BufferPosition::zero();
        self.block_anchor_col = None;
        self.block_cursor_col = None;
    }

    pub fn start_selection(&mut self, pos: BufferPosition, mode: SelectionMode) {
        self.active = true;
        self.mode = mode;
        self.anchor = pos;
        self.cursor = pos;
        if mode == SelectionMode::Block {
            self.block_anchor_col = Some(pos.col);
            self.block_cursor_col = Some(pos.col);
        }
    }

    pub fn update_cursor(&mut self, pos: BufferPosition) {
        self.cursor = pos;
        if self.mode == SelectionMode::Block {
            self.block_cursor_col = Some(pos.col);
        }
    }

    pub fn select_line(&mut self, line: usize, _line_content: &str) {
        self.active = true;
        self.mode = SelectionMode::Line;
        self.anchor = BufferPosition::new(line, 0);
        self.cursor = BufferPosition::new(line, usize::MAX);
    }

    pub fn select_all(&mut self, total_lines: usize) {
        if total_lines == 0 {
            return;
        }
        self.active = true;
        self.mode = SelectionMode::Character;
        self.anchor = BufferPosition::new(0, 0);
        self.cursor = BufferPosition::new(
            total_lines - 1,
            usize::MAX, // Will be clamped when getting text
        );
    }

    pub fn get_selected_text(&self, lines: &[String]) -> String {
        if !self.active || self.is_empty() {
            return String::new();
        }

        match self.mode {
            SelectionMode::Character => {
                let start = self.start();
                let end = self.end();

                if start.line == end.line {
                    let line = lines.get(start.line).map_or("", |l| l.as_str());
                    let end_col = end.col.min(line.len());
                    line[start.col.min(line.len())..end_col].to_string()
                } else {
                    let mut result = String::new();

                    // First line
                    if let Some(first_line) = lines.get(start.line) {
                        let s = start.col.min(first_line.len());
                        result.push_str(&first_line[s..]);
                        result.push('\n');
                    }

                    // Middle lines
                    for line_num in (start.line + 1)..end.line {
                        if let Some(line) = lines.get(line_num) {
                            result.push_str(line);
                            result.push('\n');
                        }
                    }

                    // Last line
                    if let Some(last_line) = lines.get(end.line) {
                        let e = end.col.min(last_line.len());
                        result.push_str(&last_line[..e]);
                    }

                    result
                }
            }
            SelectionMode::Line => {
                let start_line = self.anchor.line.min(self.cursor.line);
                let end_line = self.anchor.line.max(self.cursor.line);

                let mut result = String::new();
                for line_num in start_line..=end_line {
                    if let Some(line) = lines.get(line_num) {
                        result.push_str(line);
                        if line_num < end_line {
                            result.push('\n');
                        }
                    }
                }
                result
            }
            SelectionMode::Block => {
                let start_line = self.anchor.line.min(self.cursor.line);
                let end_line = self.anchor.line.max(self.cursor.line);
                let start_col = self.block_anchor_col.unwrap_or(0).min(self.block_cursor_col.unwrap_or(0));
                let end_col = self.block_anchor_col.unwrap_or(0).max(self.block_cursor_col.unwrap_or(0));

                let mut result = String::new();
                for line_num in start_line..=end_line {
                    if let Some(line) = lines.get(line_num) {
                        let s = start_col.min(line.len());
                        let e = end_col.min(line.len());
                        result.push_str(&line[s..e]);
                        if line_num < end_line {
                            result.push('\n');
                        }
                    }
                }
                result
            }
        }
    }

    pub fn contains(&self, pos: BufferPosition) -> bool {
        if !self.active {
            return false;
        }
        let start = self.start();
        let end = self.end();
        pos >= start && pos <= end
    }

    pub fn expand_to_word(&mut self, lines: &[String]) {
        if !self.active {
            return;
        }

        let line_idx = self.cursor.line;
        if line_idx >= lines.len() {
            return;
        }

        let line = &lines[line_idx];
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor.col.min(chars.len());

        // Move backward to word boundary
        while col > 0 && col <= chars.len() && !chars[col - 1].is_whitespace() {
            col -= 1;
        }
        let word_start = BufferPosition::new(line_idx, col);

        // Move forward to word boundary
        let mut end_col = self.cursor.col.min(chars.len());
        while end_col < chars.len() && !chars[end_col].is_whitespace() {
            end_col += 1;
        }
        let word_end = BufferPosition::new(line_idx, end_col);

        self.anchor = word_start;
        self.cursor = word_end;
    }

    pub fn expand_to_line(&mut self) {
        if !self.active {
            return;
        }
        self.mode = SelectionMode::Line;
        self.anchor = BufferPosition::new(self.anchor.line, 0);
        self.cursor = BufferPosition::new(self.cursor.line, usize::MAX);
    }

    pub fn line_ranges(&self) -> Vec<(usize, usize, usize, usize)> {
        // Returns (line, start_col, end_col, is_full_line) tuples
        if !self.active {
            return Vec::new();
        }

        let start = self.start();
        let end = self.end();

        let mut ranges = Vec::new();
        for line_num in start.line..=end.line {
            let s = if line_num == start.line { start.col } else { 0 };
            let e = if line_num == end.line {
                end.col
            } else {
                usize::MAX
            };
            ranges.push((line_num, s, e, 0));
        }
        ranges
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}
