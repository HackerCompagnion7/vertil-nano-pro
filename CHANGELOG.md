# Vertil Nano Pro - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-06-06

### Added

#### Editor Core
- Text buffer with line-based storage
- Cursor navigation (character, word, line, page, buffer)
- Undo/Redo with history stack (up to 1000 entries)
- Multiple tabs for editing several files
- Auto-indentation on Enter
- Indent/dedent with Tab/Shift+Tab
- Line duplication
- Line moving (up/down)
- Comment toggling
- File detection and encoding (UTF-8)

#### Modern Selection
- Character selection with Shift+Arrow keys
- Line selection mode
- Block (column) selection mode
- Mouse drag selection
- Select line, function, class, and entire file
- Selection persists during scrolling
- Auto-scroll during selection

#### Clipboard
- Clipboard history (up to 100 entries)
- Copy line, block, function, class, or entire file
- System clipboard integration (xclip, wl-copy, pbcopy, termux-clipboard-set)
- Paste from history
- Clipboard entry preview

#### Syntax Highlighting
- Tree-sitter integration for accurate parsing
- Language support: Python, JavaScript, TypeScript, Go, Rust, C, C++
- Fallback highlighting for all file types
- Token types: keyword, string, comment, function, type, number, operator, variable, constant, property, punctuation, tag, attribute, escape
- Catppuccin Mocha dark theme
- Catppuccin Latte light theme
- Custom theme support via TOML files

#### Project Explorer
- Side panel with file tree
- Expand/collapse directories
- Create, rename, delete, and move files
- Project root auto-detection
- Smart directory filtering (node_modules, target, .git, etc.)
- File search by name
- Keyboard navigation within explorer

#### Search & Replace
- In-file search with live preview
- Search next/previous
- Case-sensitive, whole-word, and regex modes
- Global search across entire project
- Global replace with file filter
- Result count limiting for performance

#### LSP Client
- Pre-configured LSP servers for 7 languages
- Language detection (Python: pylsp, JS/TS: typescript-language-server, Go: gopls, Rust: rust-analyzer, C/C++: clangd)
- LSP server availability detection
- Keyword completion as fallback
- Diagnostic collection architecture
- Completion item types

#### Git Integration
- Repository discovery and opening
- Status display (new, modified, deleted, untracked, conflicted)
- Diff generation (additions, deletions, context)
- Commit creation (stage all + commit)
- Current branch detection
- Commit log viewing
- Formatted status and diff output

#### Configuration
- TOML-based configuration file
- XDG config directory support
- Load-or-create pattern
- Theme switching (dark/light)
- Configurable tab size, indent style, line numbers, soft wrap
- Auto-save option
- LSP and Git enable/disable flags
- Custom keybinding definitions

#### Terminal UI
- Crossterm-based rendering engine
- Tab bar with active tab highlighting
- Line numbers with dynamic width
- Status bar (filename, language, cursor position)
- Shortcut bar with keybinding hints
- Splash screen on startup
- Dialog rendering for modals
- Mouse support (click, drag, scroll)
- Horizontal scrolling for long lines
- Unicode width handling

#### CLI
- `nanopro <file>` — Open file for editing
- `nanopro --about` — Show editor information
- `nanopro --version` / `nanopro -v` — Show version
- Multiple file arguments open as tabs

#### Packaging
- Build script for Linux (release/debug)
- Install script for system-wide installation
- Uninstall script
- Termux build script
- Termux package metadata (PKGBUILD format)
- Portable tarball creation for Termux

#### Documentation
- README with full feature documentation
- ARCHITECTURE.md with module details
- KEYBINDINGS.md with complete keybinding reference
- CHANGELOG.md (this file)
- MIT LICENSE

### Design Decisions
- **Rust** chosen for performance, safety, and single-binary output
- **Line-based buffer** over rope for simplicity (works well up to ~50k lines)
- **Ctrl-based shortcuts** like Nano (not modal like Vim)
- **Tree-sitter** for syntax highlighting (incremental, accurate, multi-language)
- **libgit2** for Git (no external git command dependency)
- **crossterm** for terminal control (cross-platform, well-maintained)
