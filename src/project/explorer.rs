use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ProjectEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

pub struct ProjectExplorer {
    root: Option<PathBuf>,
    entries: Vec<ProjectEntry>,
    selected_index: usize,
    visible: bool,
    width: usize,
}

impl ProjectExplorer {
    pub fn new() -> Self {
        Self {
            root: None,
            entries: Vec::new(),
            selected_index: 0,
            visible: false,
            width: 25,
        }
    }

    pub fn set_root(&mut self, path: &Path) {
        self.root = Some(path.to_path_buf());
        self.refresh();
    }

    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    pub fn refresh(&mut self) {
        if self.root.is_none() {
            return;
        }

        self.entries.clear();
        let root = self.root.clone().unwrap();

        if let Some(name) = root.file_name().and_then(|n| n.to_str()) {
            self.entries.push(ProjectEntry {
                path: root.to_path_buf(),
                name: name.to_string(),
                is_dir: true,
                depth: 0,
                expanded: true,
            });
        }

        self.load_directory(&root, 1);
    }

    fn load_directory(&mut self, dir: &Path, depth: usize) {
        if depth > 10 {
            return;
        }

        let mut entries: Vec<std::fs::DirEntry> = match fs::read_dir(dir) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
            Err(_) => return,
        };

        entries.sort_by(|a, b| {
            let a_dir = a.file_type().map_or(false, |t| t.is_dir());
            let b_dir = b.file_type().map_or(false, |t| t.is_dir());
            match (a_dir, b_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files and common ignored directories
            if name.starts_with('.') {
                continue;
            }
            if matches!(name.as_str(), "node_modules" | "target" | "__pycache__" | ".git" | "build" | "dist" | ".cache" | "vendor" | ".venv" | "venv") {
                continue;
            }

            let path = entry.path();
            let is_dir = entry.file_type().map_or(false, |t| t.is_dir());

            self.entries.push(ProjectEntry {
                path: path.clone(),
                name,
                is_dir,
                depth,
                expanded: false,
            });
        }
    }

    pub fn toggle_expand(&mut self, index: usize) -> Option<PathBuf> {
        let entry = self.entries.get(index)?.clone();

        if !entry.is_dir {
            return Some(entry.path.clone());
        }

        let was_expanded = entry.expanded;
        let dir_path = entry.path.clone();
        let dir_depth = entry.depth;
        self.entries[index].expanded = !was_expanded;

        if was_expanded {
            // Collapse: remove child entries
            let mut remove_indices = Vec::new();

            for (i, e) in self.entries.iter().enumerate() {
                if i <= index {
                    continue;
                }
                if e.depth <= dir_depth {
                    break;
                }
                if e.path.starts_with(&dir_path) {
                    remove_indices.push(i);
                }
            }

            // Remove in reverse order to maintain indices
            for i in remove_indices.into_iter().rev() {
                self.entries.remove(i);
            }
        } else {
            // Expand: insert child entries
            let child_depth = dir_depth + 1;

            let mut new_entries = Vec::new();
            if let Ok(rd) = fs::read_dir(&dir_path) {
                let mut children: Vec<std::fs::DirEntry> = rd.filter_map(|e| e.ok()).collect();
                children.sort_by(|a, b| {
                    let a_dir = a.file_type().map_or(false, |t| t.is_dir());
                    let b_dir = b.file_type().map_or(false, |t| t.is_dir());
                    match (a_dir, b_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.file_name().cmp(&b.file_name()),
                    }
                });

                for child in children {
                    let name = child.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') {
                        continue;
                    }
                    if matches!(name.as_str(), "node_modules" | "target" | "__pycache__" | ".git" | "build" | "dist" | ".cache" | "vendor" | ".venv" | "venv") {
                        continue;
                    }

                    let path = child.path();
                    let is_dir = child.file_type().map_or(false, |t| t.is_dir());

                    new_entries.push(ProjectEntry {
                        path,
                        name,
                        is_dir,
                        depth: child_depth,
                        expanded: false,
                    });
                }
            }

            for (offset, entry) in new_entries.into_iter().enumerate() {
                self.entries.insert(index + 1 + offset, entry);
            }
        }

        None
    }

    pub fn select_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn select_down(&mut self) {
        if self.selected_index + 1 < self.entries.len() {
            self.selected_index += 1;
        }
    }

    pub fn selected(&self) -> Option<&ProjectEntry> {
        self.entries.get(self.selected_index)
    }

    pub fn selected_path(&self) -> Option<PathBuf> {
        self.entries.get(self.selected_index).map(|e| e.path.clone())
    }

    pub fn entries(&self) -> &[ProjectEntry] {
        &self.entries
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn set_selected_index(&mut self, idx: usize) {
        self.selected_index = idx.min(self.entries.len().saturating_sub(1));
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;
    }

    pub fn create_file(&mut self, path: &Path) -> Result<(), String> {
        if path.exists() {
            return Err(format!("File already exists: {}", path.display()));
        }
        fs::write(path, "").map_err(|e| format!("Failed to create file: {}", e))?;
        self.refresh();
        Ok(())
    }

    pub fn create_directory(&mut self, path: &Path) -> Result<(), String> {
        if path.exists() {
            return Err(format!("Directory already exists: {}", path.display()));
        }
        fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {}", e))?;
        self.refresh();
        Ok(())
    }

    pub fn rename(&mut self, old_path: &Path, new_path: &Path) -> Result<(), String> {
        fs::rename(old_path, new_path).map_err(|e| format!("Failed to rename: {}", e))?;
        self.refresh();
        Ok(())
    }

    pub fn delete(&mut self, path: &Path) -> Result<(), String> {
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| format!("Failed to delete directory: {}", e))?;
        } else {
            fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))?;
        }
        self.refresh();
        Ok(())
    }

    pub fn move_entry(&mut self, src: &Path, dest_dir: &Path) -> Result<(), String> {
        let file_name = src.file_name().ok_or("Invalid source path")?;
        let dest = dest_dir.join(file_name);
        fs::rename(src, &dest).map_err(|e| format!("Failed to move: {}", e))?;
        self.refresh();
        Ok(())
    }

    pub fn find_project_root(path: &Path) -> Option<PathBuf> {
        let mut current = path;

        loop {
            let indicators = [
                ".git",
                "Cargo.toml",
                "package.json",
                "go.mod",
                "pyproject.toml",
                "setup.py",
                "CMakeLists.txt",
                "Makefile",
                ".project",
            ];

            for indicator in &indicators {
                if current.join(indicator).exists() {
                    return Some(current.to_path_buf());
                }
            }

            current = current.parent()?;
        }
    }

    pub fn search_files(&self, query: &str) -> Vec<PathBuf> {
        let root = match &self.root {
            Some(r) => r,
            None => return Vec::new(),
        };

        let lower_query = query.to_lowercase();
        let mut results = Vec::new();

        for entry in WalkDir::new(root)
            .follow_links(false)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.to_lowercase().contains(&lower_query) {
                results.push(entry.into_path());
            }
        }

        results
    }
}

impl Default for ProjectExplorer {
    fn default() -> Self {
        Self::new()
    }
}
