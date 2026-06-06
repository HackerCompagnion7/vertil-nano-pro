#![allow(dead_code)]
use regex::Regex;
use std::path::{Path, PathBuf};
use crate::editor::BufferPosition;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub line: usize,
    pub col: usize,
    pub text: String,
    pub path: Option<PathBuf>,
}

pub struct SearchEngine {
    case_sensitive: bool,
    whole_word: bool,
    regex_mode: bool,
    last_pattern: Option<String>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            regex_mode: false,
            last_pattern: None,
        }
    }

    pub fn search_in_buffer(&self, pattern: &str, lines: &[String], start_from: Option<BufferPosition>) -> Vec<SearchResult> {
        if pattern.is_empty() {
            return Vec::new();
        }

        let search_pattern = if self.whole_word {
            format!(r"\b{}\b", regex::escape(pattern))
        } else if self.regex_mode {
            pattern.to_string()
        } else {
            regex::escape(pattern)
        };

        let re = if self.case_sensitive {
            Regex::new(&search_pattern)
        } else {
            Regex::new(&format!("(?i){}", search_pattern))
        };

        let re = match re {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let start_line = start_from.map_or(0, |p| p.line);
        let start_col = start_from.map_or(0, |p| p.col);
        let mut results = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            if line_idx < start_line {
                continue;
            }

            for mat in re.find_iter(line) {
                let col = mat.start();
                if line_idx == start_line && col < start_col {
                    continue;
                }

                results.push(SearchResult {
                    line: line_idx,
                    col,
                    text: mat.as_str().to_string(),
                    path: None,
                });
            }
        }

        results
    }

    pub fn search_next(&mut self, pattern: &str, lines: &[String], from: BufferPosition) -> Option<SearchResult> {
        self.last_pattern = Some(pattern.to_string());
        let results = self.search_in_buffer(pattern, lines, Some(from));
        results.into_iter().next()
    }

    pub fn search_prev(&mut self, pattern: &str, lines: &[String], from: BufferPosition) -> Option<SearchResult> {
        self.last_pattern = Some(pattern.to_string());
        let results = self.search_in_buffer(pattern, lines, None);

        // Find the last result before the current position
        results.into_iter().rev().find(|r| {
            r.line < from.line || (r.line == from.line && r.col < from.col)
        })
    }

    pub fn replace_in_text(&self, text: &str, pattern: &str, replacement: &str) -> Result<String, String> {
        let search_pattern = if self.whole_word {
            format!(r"\b{}\b", regex::escape(pattern))
        } else if self.regex_mode {
            pattern.to_string()
        } else {
            regex::escape(pattern)
        };

        let re = if self.case_sensitive {
            Regex::new(&search_pattern)
        } else {
            Regex::new(&format!("(?i){}", search_pattern))
        };

        let re = re.map_err(|e| format!("Invalid pattern: {}", e))?;
        Ok(re.replace_all(text, replacement).to_string())
    }

    pub fn replace_in_buffer(&self, lines: &[String], pattern: &str, replacement: &str) -> Vec<(usize, String)> {
        let search_pattern = if self.whole_word {
            format!(r"\b{}\b", regex::escape(pattern))
        } else if self.regex_mode {
            pattern.to_string()
        } else {
            regex::escape(pattern)
        };

        let re = if self.case_sensitive {
            Regex::new(&search_pattern)
        } else {
            Regex::new(&format!("(?i){}", search_pattern))
        };

        let re = match re {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut changed_lines = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            if re.is_match(line) {
                let new_line = re.replace_all(line, replacement).to_string();
                changed_lines.push((line_idx, new_line));
            }
        }

        changed_lines
    }

    pub fn set_case_sensitive(&mut self, sensitive: bool) {
        self.case_sensitive = sensitive;
    }

    pub fn set_whole_word(&mut self, whole: bool) {
        self.whole_word = whole;
    }

    pub fn set_regex_mode(&mut self, regex: bool) {
        self.regex_mode = regex;
    }

    pub fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    pub fn whole_word(&self) -> bool {
        self.whole_word
    }

    pub fn regex_mode(&self) -> bool {
        self.regex_mode
    }

    pub fn last_pattern(&self) -> Option<&str> {
        self.last_pattern.as_deref()
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Global search across project files
pub struct GlobalSearch {
    search_engine: SearchEngine,
}

impl GlobalSearch {
    pub fn new() -> Self {
        Self {
            search_engine: SearchEngine::new(),
        }
    }

    pub fn search_in_project(&self, pattern: &str, root: &Path, file_filter: Option<&str>) -> Vec<SearchResult> {
        let mut results = Vec::new();

        if pattern.is_empty() {
            return results;
        }

        let filter_ext = file_filter.map(|f| f.to_string());

        for entry in walkdir::WalkDir::new(root)
            .follow_links(false)
            .max_depth(20)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();

            // Skip hidden and common ignored directories
            let path_str = path.to_string_lossy();
            if path_str.contains("/node_modules/")
                || path_str.contains("/.git/")
                || path_str.contains("/target/")
                || path_str.contains("/__pycache__/")
                || path_str.contains("/.cache/")
                || path_str.contains("/vendor/")
                || path_str.contains("/build/")
                || path_str.contains("/dist/")
            {
                continue;
            }

            // Apply file filter
            if let Some(ref ext) = filter_ext {
                if !path.extension().map_or(false, |e| e == ext.as_str()) {
                    continue;
                }
            }

            // Skip binary files by checking extension
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext, "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "svg" | "woff" | "woff2" | "ttf" | "eot" | "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "pdf" | "doc" | "docx" | "xls" | "xlsx" | "exe" | "dll" | "so" | "dylib" | "o" | "a" | "class" | "jar" | "wasm" | "mp3" | "mp4" | "avi" | "mkv" | "mov") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(path) {
                let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                let buffer_results = self.search_engine.search_in_buffer(pattern, &lines, None);

                for mut result in buffer_results {
                    result.path = Some(path.to_path_buf());
                    results.push(result);
                }
            }

            // Limit results to prevent excessive memory use
            if results.len() > 5000 {
                break;
            }
        }

        results
    }

    pub fn replace_in_project(&self, pattern: &str, replacement: &str, root: &Path, file_filter: Option<&str>) -> Vec<(PathBuf, usize)> {
        let results = self.search_in_project(pattern, root, file_filter);
        let mut file_changes: Vec<(PathBuf, usize)> = Vec::new();

        // Group results by file
        let mut files: std::collections::HashMap<PathBuf, Vec<&SearchResult>> = std::collections::HashMap::new();
        for result in &results {
            if let Some(ref path) = result.path {
                files.entry(path.clone()).or_default().push(result);
            }
        }

        for (path, _results) in files {
            if let Ok(content) = std::fs::read_to_string(&path) {
                match self.search_engine.replace_in_text(&content, pattern, replacement) {
                    Ok(new_content) => {
                        let change_count = content.matches(pattern).count();
                        if let Ok(()) = std::fs::write(&path, new_content) {
                            file_changes.push((path, change_count));
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        file_changes
    }

    pub fn set_case_sensitive(&mut self, sensitive: bool) {
        self.search_engine.set_case_sensitive(sensitive);
    }

    pub fn set_whole_word(&mut self, whole: bool) {
        self.search_engine.set_whole_word(whole);
    }

    pub fn set_regex_mode(&mut self, regex: bool) {
        self.search_engine.set_regex_mode(regex);
    }
}

impl Default for GlobalSearch {
    fn default() -> Self {
        Self::new()
    }
}
