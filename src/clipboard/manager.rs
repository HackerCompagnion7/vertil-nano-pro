use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ClipEntry {
    pub id: usize,
    pub text: String,
    pub kind: ClipKind,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClipKind {
    Character,
    Line,
    Block,
    Function,
    Class,
    File,
}

impl std::fmt::Display for ClipKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipKind::Character => write!(f, "Text"),
            ClipKind::Line => write!(f, "Line"),
            ClipKind::Block => write!(f, "Block"),
            ClipKind::Function => write!(f, "Function"),
            ClipKind::Class => write!(f, "Class"),
            ClipKind::File => write!(f, "File"),
        }
    }
}

pub const CLIPBOARD_MAX_HISTORY: usize = 100;

pub struct ClipboardManager {
    history: VecDeque<ClipEntry>,
    next_id: usize,
    system_clipboard: bool,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(CLIPBOARD_MAX_HISTORY),
            next_id: 1,
            system_clipboard: false,
        }
    }

    pub fn copy(&mut self, text: &str, kind: ClipKind) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let entry = ClipEntry {
            id,
            text: text.to_string(),
            kind,
            timestamp: Instant::now(),
        };

        // Try to also copy to system clipboard
        self.copy_to_system_clipboard(text);

        self.history.push_front(entry);

        // Trim to max size
        while self.history.len() > CLIPBOARD_MAX_HISTORY {
            self.history.pop_back();
        }

        id
    }

    pub fn copy_line(&mut self, text: &str) -> usize {
        self.copy(text, ClipKind::Line)
    }

    pub fn copy_block(&mut self, text: &str) -> usize {
        self.copy(text, ClipKind::Block)
    }

    pub fn copy_function(&mut self, text: &str) -> usize {
        self.copy(text, ClipKind::Function)
    }

    pub fn copy_class(&mut self, text: &str) -> usize {
        self.copy(text, ClipKind::Class)
    }

    pub fn copy_file(&mut self, text: &str) -> usize {
        self.copy(text, ClipKind::File)
    }

    pub fn paste_latest(&self) -> Option<&str> {
        self.history.front().map(|e| e.text.as_str())
    }

    pub fn paste_by_id(&self, id: usize) -> Option<&str> {
        self.history.iter().find(|e| e.id == id).map(|e| e.text.as_str())
    }

    pub fn paste_by_index(&self, index: usize) -> Option<&str> {
        self.history.get(index).map(|e| e.text.as_str())
    }

    pub fn history(&self) -> &VecDeque<ClipEntry> {
        &self.history
    }

    pub fn history_entries(&self) -> Vec<(usize, String, ClipKind)> {
        self.history
            .iter()
            .map(|e| (e.id, e.text.clone(), e.kind))
            .collect()
    }

    pub fn recent_entries(&self, count: usize) -> Vec<&ClipEntry> {
        self.history.iter().take(count).collect()
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }

    pub fn entry_count(&self) -> usize {
        self.history.len()
    }

    pub fn remove_entry(&mut self, id: usize) -> bool {
        if let Some(pos) = self.history.iter().position(|e| e.id == id) {
            self.history.remove(pos);
            true
        } else {
            false
        }
    }

    fn copy_to_system_clipboard(&self, text: &str) {
        // Try xclip (Linux), pbcopy (macOS), termux-clipboard-set (Termux)
        let attempts: Vec<(&str, Vec<&str>)> = vec![
            ("xclip", vec!["-selection", "clipboard"]),
            ("xsel", vec!["--clipboard", "--input"]),
            ("pbcopy", vec![]),
            ("termux-clipboard-set", vec![]),
            ("wl-copy", vec![]),
        ];

        for (cmd, args) in &attempts {
            if let Ok(mut child) = std::process::Command::new(cmd)
                .args(args)
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(text.as_bytes());
                    let _ = stdin.flush();
                }
                let _ = child.wait();
                return;
            }
        }
    }

    fn paste_from_system_clipboard(&self) -> Option<String> {
        let attempts: Vec<(&str, Vec<&str>)> = vec![
            ("xclip", vec!["-selection", "clipboard", "-o"]),
            ("xsel", vec!["--clipboard", "--output"]),
            ("pbpaste", vec![]),
            ("termux-clipboard-get", vec![]),
            ("wl-paste", vec![]),
        ];

        for (cmd, args) in &attempts {
            if let Ok(output) = std::process::Command::new(cmd)
                .args(args)
                .output()
            {
                if output.status.success() {
                    return Some(String::from_utf8_lossy(&output.stdout).to_string());
                }
            }
        }

        None
    }

    pub fn get_system_clipboard(&self) -> Option<String> {
        self.paste_from_system_clipboard()
    }

    pub fn format_entry_preview(text: &str, max_len: usize) -> String {
        let single_line: String = text.lines().next().unwrap_or("").chars().take(max_len).collect();
        let extra = if text.lines().count() > 1 { "..." } else { "" };
        format!("{}{}", single_line, extra)
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}
