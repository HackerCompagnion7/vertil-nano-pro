use crate::editor::cursor::Cursor;
use unicode_width::UnicodeWidthStr;

pub struct View {
    pub scroll_y: usize,
    pub scroll_x: usize,
    pub visible_width: usize,
    pub visible_height: usize,
    pub line_number_width: usize,
    #[allow(dead_code)]
    pub soft_wrap: bool,
}

#[allow(dead_code)]
impl View {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            scroll_y: 0,
            scroll_x: 0,
            visible_width: width,
            visible_height: height.saturating_sub(3),
            line_number_width: 4,
            soft_wrap: false,
        }
    }

    pub fn ensure_cursor_visible(&mut self, cursor: &Cursor, lines: &[String]) {
        if cursor.line < self.scroll_y {
            self.scroll_y = cursor.line;
        } else if cursor.line >= self.scroll_y + self.visible_height {
            self.scroll_y = cursor.line - self.visible_height + 1;
        }

        let line = lines.get(cursor.line).map_or("", |l| l.as_str());
        let col = cursor.col.min(line.len());
        let before_cursor = &line[..col];
        let cursor_visual_col = UnicodeWidthStr::width(before_cursor);

        let content_start = self.line_number_width + 1;
        let content_width = self.visible_width.saturating_sub(content_start);

        if cursor_visual_col < self.scroll_x {
            self.scroll_x = if cursor_visual_col > 4 {
                cursor_visual_col - 4
            } else {
                0
            };
        } else if cursor_visual_col >= self.scroll_x + content_width {
            self.scroll_x = cursor_visual_col - content_width + 4;
        }
    }

    pub fn visible_line_range(&self) -> (usize, usize) {
        let start = self.scroll_y;
        let end = start + self.visible_height;
        (start, end)
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_y = self.scroll_y.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize, total_lines: usize) {
        self.scroll_y = (self.scroll_y + amount).min(total_lines.saturating_sub(1));
    }

    pub fn scroll_left(&mut self, amount: usize) {
        self.scroll_x = self.scroll_x.saturating_sub(amount);
    }

    pub fn scroll_right(&mut self, amount: usize) {
        self.scroll_x += amount;
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.visible_width = width;
        self.visible_height = height.saturating_sub(3);
    }

    pub fn update_line_number_width(&mut self, total_lines: usize) {
        self.line_number_width = if total_lines < 10 {
            2
        } else if total_lines < 100 {
            3
        } else if total_lines < 1000 {
            4
        } else if total_lines < 10000 {
            5
        } else {
            6
        };
    }

    pub fn content_start_col(&self) -> usize {
        self.line_number_width + 1
    }

    pub fn content_width(&self) -> usize {
        self.visible_width.saturating_sub(self.content_start_col())
    }

    pub fn center_on_line(&mut self, line: usize, total_lines: usize) {
        let half = self.visible_height / 2;
        if line >= half {
            self.scroll_y = line - half;
        } else {
            self.scroll_y = 0;
        }
        let max_scroll = total_lines.saturating_sub(self.visible_height);
        if self.scroll_y > max_scroll {
            self.scroll_y = max_scroll;
        }
    }
}
