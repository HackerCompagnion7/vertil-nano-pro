use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};

use crate::editor::{Buffer, BufferPosition, Cursor, Selection, SelectionMode, TabManager, View};
use crate::syntax::highlight::{HighlightEngine, HighlightSpan};
use crate::config::settings::Settings;

pub struct Renderer<'a> {
    stdout: io::Stdout,
    pub settings: &'a Settings,
    pub highlight_engine: HighlightEngine,
}

impl<'a> Renderer<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        Self {
            stdout: io::stdout(),
            settings,
            highlight_engine: HighlightEngine::new(),
        }
    }

    pub fn init_terminal(&mut self) -> io::Result<()> {
        execute!(self.stdout, EnterAlternateScreen, EnableMouseCapture)?;
        crossterm::terminal::enable_raw_mode()?;
        execute!(self.stdout, Hide)?;
        Ok(())
    }

    pub fn restore_terminal(&mut self) -> io::Result<()> {
        execute!(self.stdout, Show, DisableMouseCapture, LeaveAlternateScreen)?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn render(
        &mut self,
        tab_manager: &TabManager,
        view: &mut View,
        cursor: &Cursor,
        selection: &Selection,
        message: Option<&str>,
    ) -> io::Result<()> {
        let tab = match tab_manager.active() {
            Some(t) => t,
            None => return self.render_empty(),
        };

        let lines = tab.buffer.lines();
        let (term_cols, term_rows) = crossterm::terminal::size()?;
        view.resize(term_cols as usize, term_rows as usize);
        view.update_line_number_width(lines.len());
        view.ensure_cursor_visible(cursor, lines);

        let (vis_start, vis_end) = view.visible_line_range();

        // Render tab bar
        self.render_tab_bar(tab_manager, term_cols as usize)?;

        // Render editor content
        let editor_top = 1u16;
        let content_height = (term_rows as usize).saturating_sub(3); // tabs + status + shortcut bar

        for row in 0..content_height {
            let line_idx = vis_start + row;
            queue!(self.stdout, MoveTo(0, editor_top + row as u16))?;

            if line_idx < lines.len() {
                // Line number
                let line_num_str = format!(
                    "{:>width$} ",
                    line_idx + 1,
                    width = view.line_number_width
                );
                queue!(
                    self.stdout,
                    SetForegroundColor(self.settings.theme.line_number_fg),
                    SetBackgroundColor(self.settings.theme.background),
                    Print(&line_num_str)
                )?;

                // Line content
                let line = &lines[line_idx];
                let highlights = self.highlight_engine.highlight(line, tab.buffer.language_id());

                // Handle horizontal scroll
                let display_line = self.apply_horizontal_scroll(line, view.scroll_x, view.content_width());

                // Check if this line has selection
                let selected = selection.active && !selection.is_empty();
                let sel_start = if selected { selection.start() } else { BufferPosition::zero() };
                let sel_end = if selected { selection.end() } else { BufferPosition::zero() };

                let in_selection = selected && line_idx >= sel_start.line && line_idx <= sel_end.line;

                if in_selection {
                    self.render_line_with_selection(
                        &display_line,
                        &highlights,
                        line_idx,
                        selection,
                        view.scroll_x,
                        view.content_width(),
                    )?;
                } else {
                    self.render_highlighted_line(&display_line, &highlights)?;
                }

                // Clear rest of line
                let used = view.content_start_col() + display_line.len();
                if used < term_cols as usize {
                    queue!(
                        self.stdout,
                        SetBackgroundColor(self.settings.theme.background),
                        Print(" ".repeat(term_cols as usize - used))
                    )?;
                }
            } else {
                // Empty line
                queue!(
                    self.stdout,
                    SetForegroundColor(self.settings.theme.line_number_fg),
                    SetBackgroundColor(self.settings.theme.background),
                    Print(format!(
                        "{:>width$} ",
                        "~",
                        width = view.line_number_width
                    )),
                    SetBackgroundColor(self.settings.theme.background),
                    Print(" ".repeat(term_cols as usize - view.line_number_width - 1))
                )?;
            }
        }

        // Render status bar
        self.render_status_bar(tab_manager, cursor, lines.len(), term_cols as usize, term_rows)?;

        // Render shortcut bar
        self.render_shortcut_bar(term_cols as usize, term_rows, message)?;

        // Position cursor
        let cursor_visual_line = (cursor.line.saturating_sub(view.scroll_y)) as u16;
        let line = lines.get(cursor.line).map_or("", |l| l.as_str());
        let cursor_visual_col = if cursor.col <= line.len() {
            unicode_width::UnicodeWidthStr::width(line[..cursor.col].as_ref())
        } else {
            line.len()
        };

        let screen_col = (view.content_start_col() + cursor_visual_col.saturating_sub(view.scroll_x)) as u16;
        queue!(self.stdout, MoveTo(screen_col, editor_top + cursor_visual_line), Show)?;

        self.stdout.flush()?;
        Ok(())
    }

    fn apply_horizontal_scroll(&self, line: &str, scroll_x: usize, max_width: usize) -> String {
        if scroll_x == 0 {
            return line.to_string();
        }

        let chars: Vec<char> = line.chars().collect();
        let mut visual_col = 0;
        let mut start_idx = 0;
        let mut result_chars = Vec::new();

        for (i, &ch) in chars.iter().enumerate() {
            let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if visual_col + w > scroll_x && start_idx == 0 {
                start_idx = i;
                visual_col = 0;
            }
            if i >= start_idx {
                if visual_col + w > max_width {
                    break;
                }
                result_chars.push(ch);
                visual_col += w;
            } else {
                visual_col += w;
            }
        }

        result_chars.into_iter().collect()
    }

    fn render_highlighted_line(&mut self, line: &str, highlights: &[HighlightSpan]) -> io::Result<()> {
        queue!(self.stdout, SetBackgroundColor(self.settings.theme.background))?;

        if highlights.is_empty() {
            queue!(
                self.stdout,
                SetForegroundColor(self.settings.theme.foreground),
                Print(line)
            )?;
        } else {
            let chars: Vec<char> = line.chars().collect();
            let mut col = 0;
            let mut highlight_idx = 0;

            while col < chars.len() {
                // Find applicable highlight
                let color = if highlight_idx < highlights.len() {
                    let hl = &highlights[highlight_idx];
                    if col >= hl.start && col < hl.end {
                        if col + 1 >= hl.end {
                            highlight_idx += 1;
                        }
                        self.token_type_to_color(hl.token_type)
                    } else if col >= hl.end {
                        highlight_idx += 1;
                        continue;
                    } else {
                        self.settings.theme.foreground
                    }
                } else {
                    self.settings.theme.foreground
                };

                queue!(
                    self.stdout,
                    SetForegroundColor(color),
                    Print(chars[col])
                )?;
                col += 1;
            }
        }

        Ok(())
    }

    fn render_line_with_selection(
        &mut self,
        line: &str,
        highlights: &[HighlightSpan],
        line_idx: usize,
        selection: &Selection,
        scroll_x: usize,
        _content_width: usize,
    ) -> io::Result<()> {
        let sel_start = selection.start();
        let sel_end = selection.end();
        let line_len = line.len();

        let (sel_start_col, sel_end_col) = if line_idx == sel_start.line && line_idx == sel_end.line {
            (sel_start.col, sel_end.col.min(line_len))
        } else if line_idx == sel_start.line {
            (sel_start.col, line_len)
        } else if line_idx == sel_end.line {
            (0, sel_end.col.min(line_len))
        } else {
            (0, line_len)
        };

        let chars: Vec<char> = line.chars().collect();
        let mut col = 0;
        let mut highlight_idx = 0;

        queue!(self.stdout, SetBackgroundColor(self.settings.theme.background))?;

        while col < chars.len() {
            let in_sel = col >= sel_start_col && col < sel_end_col;

            let fg = if highlight_idx < highlights.len() {
                let hl = &highlights[highlight_idx];
                if col >= hl.start && col < hl.end {
                    if col + 1 >= hl.end {
                        highlight_idx += 1;
                    }
                    self.token_type_to_color(hl.token_type)
                } else if col >= hl.end {
                    highlight_idx += 1;
                    continue;
                } else {
                    self.settings.theme.foreground
                }
            } else {
                self.settings.theme.foreground
            };

            let bg = if in_sel {
                self.settings.theme.selection_bg
            } else {
                self.settings.theme.background
            };

            queue!(
                self.stdout,
                SetForegroundColor(fg),
                SetBackgroundColor(bg),
                Print(chars[col])
            )?;
            col += 1;
        }

        // Fill selection to end of line if needed
        if sel_end_col >= line_len && line_idx >= sel_start.line && line_idx <= sel_end.line {
            // Line is fully selected to the end - visual cue already applied
        }

        Ok(())
    }

    fn token_type_to_color(&self, token_type: u32) -> Color {
        // Tree-sitter highlight capture mappings
        match token_type {
            1 => self.settings.theme.keyword,
            2 => self.settings.theme.string_literal,
            3 => self.settings.theme.comment,
            4 => self.settings.theme.function_,
            5 => self.settings.theme.type_,
            6 => self.settings.theme.number,
            7 => self.settings.theme.operator,
            8 => self.settings.theme.variable,
            9 => self.settings.theme.constant,
            10 => self.settings.theme.property,
            11 => self.settings.theme.punctuation,
            12 => self.settings.theme.tag,
            13 => self.settings.theme.attribute,
            14 => self.settings.theme.escape_,
            _ => self.settings.theme.foreground,
        }
    }

    fn render_tab_bar(&mut self, tab_manager: &TabManager, width: usize) -> io::Result<()> {
        queue!(self.stdout, MoveTo(0, 0))?;
        queue!(
            self.stdout,
            SetBackgroundColor(self.settings.theme.tab_bar_bg),
            SetForegroundColor(self.settings.theme.tab_bar_fg)
        )?;

        let tab_names = tab_manager.tab_names();
        let active_idx = tab_manager.active_index();
        let mut col = 0;

        for (i, (name, _modified, _id)) in tab_names.iter().enumerate() {
            let display = format!(" {} ", name);
            let max_display = if display.len() > 20 { 20 } else { display.len() };
            let truncated: String = display.chars().take(max_display).collect();

            if i == active_idx {
                queue!(
                    self.stdout,
                    SetForegroundColor(self.settings.theme.tab_active_fg),
                    SetBackgroundColor(self.settings.theme.tab_active_bg),
                    Print(&truncated)
                )?;
            } else {
                queue!(
                    self.stdout,
                    SetForegroundColor(self.settings.theme.tab_bar_fg),
                    SetBackgroundColor(self.settings.theme.tab_bar_bg),
                    Print(&truncated)
                )?;
            }
            col += truncated.len();
        }

        // Fill rest of tab bar
        if col < width {
            queue!(
                self.stdout,
                SetBackgroundColor(self.settings.theme.tab_bar_bg),
                Print(" ".repeat(width - col))
            )?;
        }

        Ok(())
    }

    fn render_status_bar(
        &mut self,
        tab_manager: &TabManager,
        cursor: &Cursor,
        total_lines: usize,
        width: usize,
        _term_rows: u16,
    ) -> io::Result<()> {
        let status_row = crossterm::terminal::size()?.1.saturating_sub(2);
        queue!(self.stdout, MoveTo(0, status_row))?;
        queue!(
            self.stdout,
            SetBackgroundColor(self.settings.theme.status_bar_bg),
            SetForegroundColor(self.settings.theme.status_bar_fg)
        )?;

        let tab = tab_manager.active();
        let (filename, lang, modified) = match tab {
            Some(t) => (t.buffer.filename().to_string(), t.buffer.language_id().to_string(), t.buffer.is_dirty()),
            None => ("[No File]".to_string(), "plaintext".to_string(), false),
        };

        let mod_indicator = if modified { " [Modified]" } else { "" };
        let left = format!(" {} {}{} ", filename, lang, mod_indicator);
        let right = format!(" Ln {}, Col {} ", cursor.line + 1, cursor.col + 1);

        let left_len = left.len();
        let right_len = right.len();

        queue!(self.stdout, Print(&left))?;

        if left_len + right_len < width {
            queue!(
                self.stdout,
                Print(" ".repeat(width - left_len - right_len))
            )?;
        }

        queue!(self.stdout, Print(&right))?;

        Ok(())
    }

    fn render_shortcut_bar(
        &mut self,
        width: usize,
        _term_rows: u16,
        message: Option<&str>,
    ) -> io::Result<()> {
        let shortcut_row = crossterm::terminal::size()?.1.saturating_sub(1);
        queue!(self.stdout, MoveTo(0, shortcut_row))?;
        queue!(
            self.stdout,
            SetBackgroundColor(self.settings.theme.shortcut_bar_bg),
            SetForegroundColor(self.settings.theme.shortcut_bar_fg)
        )?;

        if let Some(msg) = message {
            let display = format!(" {}", msg);
            queue!(self.stdout, Print(display))?;
            queue!(
                self.stdout,
                Print(" ".repeat(width.saturating_sub(msg.len() + 1)))
            )?;
        } else {
            let shortcuts = [
                ("^G Help", "^O Save", "^W Search", "^K Cut", "^U Paste", "^T Tree", "^Q Quit"),
            ];

            let mut display = String::new();
            for (i, sc) in shortcuts[0].iter().enumerate() {
                if i > 0 {
                    display.push_str("  ");
                }
                display.push_str(sc);
            }

            queue!(self.stdout, Print(&display))?;
            let remaining = width.saturating_sub(display.len());
            if remaining > 0 {
                queue!(self.stdout, Print(" ".repeat(remaining)))?;
            }
        }

        Ok(())
    }

    fn render_empty(&mut self) -> io::Result<()> {
        queue!(self.stdout, Clear(ClearType::All))?;
        queue!(self.stdout, MoveTo(0, 0))?;
        queue!(
            self.stdout,
            SetForegroundColor(Color::Cyan),
            Print("  Vertil Nano Pro")
        )?;
        queue!(self.stdout, MoveTo(0, 1))?;
        queue!(
            self.stdout,
            SetForegroundColor(Color::White),
            Print("  Modern Terminal Code Editor")
        )?;
        queue!(self.stdout, MoveTo(0, 2))?;
        queue!(
            self.stdout,
            SetForegroundColor(Color::DarkGrey),
            Print("  Created by Ishmael Vertil")
        )?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn render_splash(&mut self) -> io::Result<()> {
        let (cols, rows) = crossterm::terminal::size()?;
        queue!(self.stdout, Clear(ClearType::All))?;

        let center_row = rows / 2;
        let lines = [
            "",
            "  ╔══════════════════════════════════════╗",
            "  ║                                      ║",
            "  ║       Vertil Nano Pro v1.0.0         ║",
            "  ║                                      ║",
            "  ║    Modern Terminal Code Editor       ║",
            "  ║                                      ║",
            "  ║    Created by Ishmael Vertil         ║",
            "  ║                                      ║",
            "  ╚══════════════════════════════════════╝",
            "",
            "  Press any key to continue...",
        ];

        let start_row = center_row.saturating_sub((lines.len() as u16) / 2);

        for (i, line) in lines.iter().enumerate() {
            queue!(
                self.stdout,
                MoveTo(0, start_row + i as u16),
                SetForegroundColor(if i >= 1 && i <= 9 {
                    Color::Cyan
                } else if i == 11 {
                    Color::DarkGrey
                } else {
                    Color::White
                }),
                SetBackgroundColor(Color::Reset),
                Print(line)
            )?;
        }

        self.stdout.flush()?;
        Ok(())
    }

    pub fn render_dialog(
        &mut self,
        title: &str,
        content: &[String],
        selected: Option<usize>,
        width: usize,
        height: usize,
    ) -> io::Result<()> {
        let (term_cols, term_rows) = crossterm::terminal::size()?;
        let start_col = (term_cols as usize).saturating_sub(width) / 2;
        let start_row = (term_rows as usize).saturating_sub(height) / 2;

        // Draw dialog box
        let border_top = format!("╔{}╗", "═".repeat(width - 2));
        let border_bottom = format!("╚{}╝", "═".repeat(width - 2));

        queue!(
            self.stdout,
            MoveTo(start_col as u16, start_row as u16),
            SetForegroundColor(Color::Cyan),
            SetBackgroundColor(Color::DarkGrey),
            Print(&border_top)
        )?;

        // Title
        let title_line = format!("║ {:^width$} ║", title, width = width - 4);
        queue!(
            self.stdout,
            MoveTo(start_col as u16, start_row as u16 + 1),
            SetForegroundColor(Color::Yellow),
            Print(&title_line)
        )?;

        let separator = format!("╟{}╢", "─".repeat(width - 2));
        queue!(
            self.stdout,
            MoveTo(start_col as u16, start_row as u16 + 2),
            SetForegroundColor(Color::Cyan),
            Print(&separator)
        )?;

        // Content
        for (i, line) in content.iter().enumerate() {
            let row = start_row as u16 + 3 + i as u16;
            if row >= start_row as u16 + height as u16 - 1 {
                break;
            }

            let display = if let Some(sel) = selected {
                if i == sel {
                    format!("║ ► {:width$} ║", line, width = width - 6)
                } else {
                    format!("║   {:width$} ║", line, width = width - 6)
                }
            } else {
                format!("║ {:width$} ║", line, width = width - 4)
            };

            let fg = if let Some(sel) = selected {
                if i == sel {
                    Color::White
                } else {
                    Color::Grey
                }
            } else {
                Color::White
            };

            queue!(
                self.stdout,
                MoveTo(start_col as u16, row),
                SetForegroundColor(fg),
                Print(&display)
            )?;
        }

        // Bottom border
        let bottom_row = start_row as u16 + height as u16 - 1;
        queue!(
            self.stdout,
            MoveTo(start_col as u16, bottom_row),
            SetForegroundColor(Color::Cyan),
            Print(&border_bottom)
        )?;

        self.stdout.flush()?;
        Ok(())
    }

    pub fn clear_message_line(&mut self) -> io::Result<()> {
        let row = crossterm::terminal::size()?.1.saturating_sub(1);
        let cols = crossterm::terminal::size()?.0;
        queue!(
            self.stdout,
            MoveTo(0, row),
            SetBackgroundColor(self.settings.theme.shortcut_bar_bg),
            Print(" ".repeat(cols as usize))
        )?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn prompt(&mut self, prompt_text: &str) -> io::Result<()> {
        let row = crossterm::terminal::size()?.1.saturating_sub(1);
        queue!(
            self.stdout,
            MoveTo(0, row),
            SetBackgroundColor(self.settings.theme.shortcut_bar_bg),
            SetForegroundColor(Color::Yellow),
            Print(prompt_text)
        )?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn show_message(&mut self, msg: &str) -> io::Result<()> {
        let row = crossterm::terminal::size()?.1.saturating_sub(1);
        let cols = crossterm::terminal::size()?.0;
        queue!(
            self.stdout,
            MoveTo(0, row),
            SetBackgroundColor(self.settings.theme.shortcut_bar_bg),
            SetForegroundColor(Color::White),
            Print(format!(" {:width$}", msg, width = cols as usize - 1))
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}
