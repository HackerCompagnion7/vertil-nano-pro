use crate::editor::buffer::Buffer;
use std::path::PathBuf;

pub struct Tab {
    pub buffer: Buffer,
    pub id: usize,
    pub name: String,
}

impl Tab {
    pub fn new(buffer: Buffer, id: usize) -> Self {
        let name = buffer.filename().to_string();
        Self { buffer, id, name }
    }

    pub fn is_modified(&self) -> bool {
        self.buffer.is_dirty()
    }

    pub fn display_name(&self) -> String {
        if self.buffer.is_dirty() {
            format!("{}*", self.name)
        } else {
            self.name.clone()
        }
    }
}

pub struct TabManager {
    tabs: Vec<Tab>,
    active_index: usize,
    next_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_index: 0,
            next_id: 1,
        }
    }

    pub fn open_tab(&mut self, path: Option<PathBuf>) -> Result<usize, String> {
        // Check if file is already open
        if let Some(ref p) = path {
            for (i, tab) in self.tabs.iter().enumerate() {
                if tab.buffer.path() == Some(p.as_path()) {
                    self.active_index = i;
                    return Ok(tab.id);
                }
            }
        }

        let buffer = match &path {
            Some(p) => Buffer::from_file(p)?,
            None => Buffer::new(),
        };

        let id = self.next_id;
        self.next_id += 1;

        let tab = Tab::new(buffer, id);
        self.tabs.push(tab);
        self.active_index = self.tabs.len() - 1;

        Ok(id)
    }

    pub fn close_tab(&mut self, id: usize) -> Option<Tab> {
        let idx = self.tabs.iter().position(|t| t.id == id)?;
        let tab = self.tabs.remove(idx);

        if self.active_index >= self.tabs.len() && !self.tabs.is_empty() {
            self.active_index = self.tabs.len() - 1;
        }

        Some(tab)
    }

    pub fn close_current(&mut self) -> Option<Tab> {
        if self.tabs.is_empty() {
            return None;
        }
        let id = self.tabs[self.active_index].id;
        self.close_tab(id)
    }

    pub fn switch_to(&mut self, id: usize) {
        if let Some(idx) = self.tabs.iter().position(|t| t.id == id) {
            self.active_index = idx;
        }
    }

    pub fn next_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.active_index = (self.active_index + 1) % self.tabs.len();
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.active_index = if self.active_index == 0 {
            self.tabs.len() - 1
        } else {
            self.active_index - 1
        };
    }

    pub fn active(&self) -> Option<&Tab> {
        self.tabs.get(self.active_index)
    }

    pub fn active_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_index)
    }

    pub fn active_id(&self) -> Option<usize> {
        self.tabs.get(self.active_index).map(|t| t.id)
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    pub fn tab_names(&self) -> Vec<(String, bool, usize)> {
        self.tabs
            .iter()
            .map(|t| (t.display_name(), t.is_modified(), t.id))
            .collect()
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }

    pub fn set_active_index(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.active_index = idx;
        }
    }
}
