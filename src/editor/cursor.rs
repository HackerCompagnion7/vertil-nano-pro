use crate::editor::buffer::BufferPosition;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
    pub preferred_col: Option<usize>,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            line: 0,
            col: 0,
            preferred_col: None,
        }
    }

    pub fn at(line: usize, col: usize) -> Self {
        Self {
            line,
            col,
            preferred_col: None,
        }
    }

    pub fn position(&self) -> BufferPosition {
        BufferPosition::new(self.line, self.col)
    }

    pub fn set_position(&mut self, pos: BufferPosition) {
        self.line = pos.line;
        self.col = pos.col;
        self.preferred_col = None;
    }

    pub fn move_left(&mut self, lines: &[String]) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.line > 0 {
            self.line -= 1;
            self.col = lines.get(self.line).map_or(0, |l| l.len());
        }
        self.preferred_col = None;
    }

    pub fn move_right(&mut self, lines: &[String]) {
        let line_len = lines.get(self.line).map_or(0, |l| l.len());
        if self.col < line_len {
            self.col += 1;
        } else if self.line + 1 < lines.len() {
            self.line += 1;
            self.col = 0;
        }
        self.preferred_col = None;
    }

    pub fn move_up(&mut self, lines: &[String]) {
        if self.line > 0 {
            self.line -= 1;
            self.clamp_to_line(lines);
            self.apply_preferred_col(lines);
        }
    }

    pub fn move_down(&mut self, lines: &[String]) {
        if self.line + 1 < lines.len() {
            self.line += 1;
            self.clamp_to_line(lines);
            self.apply_preferred_col(lines);
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.col = 0;
        self.preferred_col = None;
    }

    pub fn move_to_line_end(&mut self, lines: &[String]) {
        self.col = lines.get(self.line).map_or(0, |l| l.len());
        self.preferred_col = None;
    }

    pub fn move_to_buffer_start(&mut self) {
        self.line = 0;
        self.col = 0;
        self.preferred_col = None;
    }

    pub fn move_to_buffer_end(&mut self, lines: &[String]) {
        if lines.is_empty() {
            self.line = 0;
            self.col = 0;
        } else {
            self.line = lines.len() - 1;
            self.col = lines.get(self.line).map_or(0, |l| l.len());
        }
        self.preferred_col = None;
    }

    pub fn move_page_up(&mut self, lines: &[String], page_size: usize) {
        let new_line = self.line.saturating_sub(page_size);
        self.line = new_line;
        self.clamp_to_line(lines);
        self.apply_preferred_col(lines);
    }

    pub fn move_page_down(&mut self, lines: &[String], page_size: usize) {
        let new_line = (self.line + page_size).min(lines.len().saturating_sub(1));
        self.line = new_line;
        self.clamp_to_line(lines);
        self.apply_preferred_col(lines);
    }

    pub fn move_word_forward(&mut self, lines: &[String]) {
        let line = lines.get(self.line);
        if let Some(line_str) = line {
            let chars: Vec<char> = line_str.chars().collect();
            let mut col = self.col;

            // Skip current word characters
            while col < chars.len() && !chars[col].is_whitespace() {
                col += 1;
            }
            // Skip whitespace
            while col < chars.len() && chars[col].is_whitespace() {
                col += 1;
            }

            if col > self.col {
                self.col = col;
            } else if self.line + 1 < lines.len() {
                self.line += 1;
                self.col = 0;
            }
        }
        self.preferred_col = None;
    }

    pub fn move_word_backward(&mut self, lines: &[String]) {
        if self.col == 0 {
            if self.line > 0 {
                self.line -= 1;
                self.col = lines.get(self.line).map_or(0, |l| l.len());
            }
            self.preferred_col = None;
            return;
        }

        let line = lines.get(self.line);
        if let Some(line_str) = line {
            let chars: Vec<char> = line_str.chars().collect();
            let mut col = self.col;

            // Skip whitespace before cursor
            while col > 0 && chars.get(col - 1).map_or(false, |c| c.is_whitespace()) {
                col -= 1;
            }
            // Skip word characters
            while col > 0 && chars.get(col - 1).map_or(false, |c| !c.is_whitespace()) {
                col -= 1;
            }

            if col < self.col {
                self.col = col;
            }
        }
        self.preferred_col = None;
    }

    pub fn clamp_to_line(&mut self, lines: &[String]) {
        let line_len = lines.get(self.line).map_or(0, |l| l.len());
        if self.col > line_len {
            self.col = line_len;
        }
    }

    fn apply_preferred_col(&mut self, lines: &[String]) {
        if self.preferred_col.is_none() {
            self.preferred_col = Some(self.col);
        }
        if let Some(pref) = self.preferred_col {
            let line_len = lines.get(self.line).map_or(0, |l| l.len());
            if pref <= line_len {
                self.col = pref;
            }
        }
    }

    pub fn save_preferred_col(&mut self) {
        self.preferred_col = Some(self.col);
    }

    pub fn goto_line(&mut self, line: usize, total_lines: usize, lines: &[String]) {
        self.line = line.min(total_lines.saturating_sub(1));
        self.clamp_to_line(lines);
        self.preferred_col = None;
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
