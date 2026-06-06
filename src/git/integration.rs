use std::path::Path;

pub struct GitIntegration {
    repo: Option<git2::Repository>,
}

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: GitFileStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GitFileStatus {
    New,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
    Untracked,
    Ignored,
    Conflicted,
}

impl std::fmt::Display for GitFileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitFileStatus::New => write!(f, "New"),
            GitFileStatus::Modified => write!(f, "Modified"),
            GitFileStatus::Deleted => write!(f, "Deleted"),
            GitFileStatus::Renamed => write!(f, "Renamed"),
            GitFileStatus::TypeChange => write!(f, "TypeChange"),
            GitFileStatus::Untracked => write!(f, "Untracked"),
            GitFileStatus::Ignored => write!(f, "Ignored"),
            GitFileStatus::Conflicted => write!(f, "Conflicted"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub content: String,
    pub kind: DiffLineKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffLineKind {
    Context,
    Addition,
    Deletion,
}

impl GitIntegration {
    pub fn new() -> Self {
        Self { repo: None }
    }

    pub fn open(&mut self, path: &Path) -> Result<(), String> {
        match git2::Repository::discover(path) {
            Ok(repo) => {
                self.repo = Some(repo);
                Ok(())
            }
            Err(e) => Err(format!("Not a git repository: {}", e.message())),
        }
    }

    pub fn is_available(&self) -> bool {
        self.repo.is_some()
    }

    pub fn status(&self) -> Result<Vec<FileStatus>, String> {
        let repo = self.repo.as_ref().ok_or("No git repository open")?;

        let mut statuses = Vec::new();
        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true);
        opts.recurse_untracked_dirs(true);

        let repo_statuses = repo
            .statuses(Some(&mut opts))
            .map_err(|e| format!("Failed to get status: {}", e.message()))?;

        for entry in repo_statuses.iter() {
            let path = entry.path().unwrap_or("unknown").to_string();
            let status = entry.status();

            let file_status = if status.is_conflicted() {
                GitFileStatus::Conflicted
            } else if status.is_index_new() || status.is_wt_new() {
                GitFileStatus::New
            } else if status.is_index_deleted() || status.is_wt_deleted() {
                GitFileStatus::Deleted
            } else if status.is_index_renamed() || status.is_wt_renamed() {
                GitFileStatus::Renamed
            } else if status.is_index_typechange() || status.is_wt_typechange() {
                GitFileStatus::TypeChange
            } else if status.is_ignored() {
                GitFileStatus::Ignored
            } else if status.is_wt_untracked() || status.is_index_new() {
                GitFileStatus::Untracked
            } else {
                GitFileStatus::Modified
            };

            statuses.push(FileStatus { path, status: file_status });
        }

        Ok(statuses)
    }

    pub fn diff(&self, file_path: &str) -> Result<Vec<DiffLine>, String> {
        let repo = self.repo.as_ref().ok_or("No git repository open")?;

        let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e.message()))?;
        let tree = head
            .peel_to_tree()
            .map_err(|e| format!("Failed to get tree: {}", e.message()))?;

        let diff = repo
            .diff_tree_to_workdir_with_index(Some(&tree), None)
            .map_err(|e| format!("Failed to get diff: {}", e.message()))?;

        let mut diff_lines = Vec::new();

        for delta in diff.deltas() {
            let new_path = delta.new_file().path().and_then(|p| p.to_str()).unwrap_or("");
            let old_path = delta.old_file().path().and_then(|p| p.to_str()).unwrap_or("");

            if new_path != file_path && old_path != file_path {
                continue;
            }

            // Get the patches for this delta
            let patch = git2::Patch::from_diff(&diff, delta.nfiles()).unwrap_or_else(|| {
                // Create an empty patch
                git2::Patch::from_diff(&diff, 0).unwrap()
            });

            if let Some(patch) = patch {
                for hunk_idx in 0..patch.num_hunks() {
                    let hunk = patch.hunk(hunk_idx)
                        .map_err(|e| format!("Failed to get hunk: {}", e.message()))?;

                    for line_idx in 0..hunk.num_lines() {
                        let line = patch.line(hunk_idx, line_idx)
                            .map_err(|e| format!("Failed to get line: {}", e.message()))?;

                        let kind = match line.origin() {
                            '+' => DiffLineKind::Addition,
                            '-' => DiffLineKind::Deletion,
                            _ => DiffLineKind::Context,
                        };

                        let content = String::from_utf8_lossy(line.content()).to_string();

                        diff_lines.push(DiffLine {
                            old_line: line.old_lineno(),
                            new_line: line.new_lineno(),
                            content,
                            kind,
                        });
                    }
                }
            }
        }

        Ok(diff_lines)
    }

    pub fn commit(&self, message: &str) -> Result<String, String> {
        let repo = self.repo.as_ref().ok_or("No git repository open")?;

        // Stage all changes
        let mut index = repo.index().map_err(|e| format!("Failed to get index: {}", e.message()))?;
        index
            .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .map_err(|e| format!("Failed to stage files: {}", e.message()))?;
        index
            .write()
            .map_err(|e| format!("Failed to write index: {}", e.message()))?;

        let tree_id = index
            .write_tree()
            .map_err(|e| format!("Failed to write tree: {}", e.message()))?;
        let tree = repo
            .find_tree(tree_id)
            .map_err(|e| format!("Failed to find tree: {}", e.message()))?;

        let sig = repo
            .signature()
            .or_else(|_| git2::Signature::now("nanopro", "nanopro@vertil.dev"))
            .map_err(|e| format!("Failed to get signature: {}", e.message()))?;

        let head = repo.head().ok();
        let parent_commit = head.as_ref().and_then(|h| h.peel_to_commit().ok());

        let parents: Vec<&git2::Commit> = if let Some(ref pc) = parent_commit {
            vec![pc]
        } else {
            vec![]
        };

        let commit_id = repo
            .commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
            .map_err(|e| format!("Failed to commit: {}", e.message()))?;

        Ok(format!("{}", commit_id))
    }

    pub fn current_branch(&self) -> Result<String, String> {
        let repo = self.repo.as_ref().ok_or("No git repository open")?;
        let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e.message()))?;

        if head.is_branch() {
            let name = head.shorthand().unwrap_or("unknown");
            Ok(name.to_string())
        } else {
            // Detached HEAD
            let target = head.target().map(|o| format!("{}", o)).unwrap_or_default();
            Ok(format!("detached: {}", &target[..7.min(target.len())]))
        }
    }

    pub fn log(&self, max_count: usize) -> Result<Vec<(String, String, String)>, String> {
        let repo = self.repo.as_ref().ok_or("No git repository open")?;

        let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e.message()))?;
        let mut revwalk = repo.revwalk().map_err(|e| format!("Failed to create revwalk: {}", e.message()))?;

        revwalk
            .push(head.target().unwrap())
            .map_err(|e| format!("Failed to push HEAD: {}", e.message()))?;

        let mut log_entries = Vec::new();

        for oid in revwalk.take(max_count) {
            let oid = oid.map_err(|e| format!("Failed to get oid: {}", e.message()))?;
            let commit = repo.find_commit(oid).map_err(|e| format!("Failed to find commit: {}", e.message()))?;

            let short_id = format!("{}", oid)[..7].to_string();
            let message = commit.message().unwrap_or("").lines().next().unwrap_or("").to_string();
            let author = commit.author().name().unwrap_or("unknown").to_string();

            log_entries.push((short_id, message, author));
        }

        Ok(log_entries)
    }

    pub fn format_status(&self) -> String {
        match self.status() {
            Ok(statuses) => {
                if statuses.is_empty() {
                    return "Working tree clean".to_string();
                }

                let mut output = String::new();
                for fs in &statuses {
                    output.push_str(&format!("  {:12} {}\n", fs.status.to_string(), fs.path));
                }
                output
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    pub fn format_diff(&self, file_path: &str) -> String {
        match self.diff(file_path) {
            Ok(lines) => {
                if lines.is_empty() {
                    return "No changes".to_string();
                }

                let mut output = String::new();
                for line in &lines {
                    let prefix = match line.kind {
                        DiffLineKind::Addition => "+",
                        DiffLineKind::Deletion => "-",
                        DiffLineKind::Context => " ",
                    };
                    output.push_str(&format!("{}{}\n", prefix, line.content.trim_end()));
                }
                output
            }
            Err(e) => format!("Error: {}", e),
        }
    }
}

impl Default for GitIntegration {
    fn default() -> Self {
        Self::new()
    }
}
