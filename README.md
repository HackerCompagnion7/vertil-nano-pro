# Vertil Nano Pro

**Modern Terminal Code Editor** — Simple like Nano, powerful for 2026

Created by **Ishmael Vertil**

---

## What is Vertil Nano Pro?

Vertil Nano Pro is a terminal-based code editor inspired by the simplicity of GNU Nano, but modernized for the needs of developers in 2026. It is not another VS Code or Neovim — it is the simplest possible editor that remains professional for modern programming.

It is designed to work especially well on:
- **Linux** (desktop and server)
- **Termux** (Android)
- **SSH** sessions
- **VPS** and remote servers
- **Low-resource machines**

## Philosophy

- **Easy to learn** — No complex key combinations to memorize
- **Instant startup** — Opens in milliseconds, not seconds
- **Simple shortcuts** — Ctrl-based, like Nano
- **Low memory usage** — Runs on 50MB RAM or less
- **No mandatory configuration** — Works from first run

## Features

### Editor
- Open, create, and save files
- Multiple tabs for working with several files
- Save As support
- Undo/Redo with full history

### Modern Selection
- Character, line, and block (column) selection
- Shift+Arrow keys for selection
- Mouse drag selection
- Select line, function, class, or entire file
- Selection persists during scrolling

### Clipboard
- Full clipboard history (up to 100 entries)
- Copy line, block, function, class, or entire file
- System clipboard integration (xclip, wl-copy, pbcopy, termux-clipboard-set)
- Paste from history

### Syntax Highlighting
- Tree-sitter based highlighting
- Support for: Python, JavaScript, TypeScript, Go, Rust, C, C++
- Catppuccin-based dark and light themes
- Fallback highlighting for all file types

### Project Explorer
- Side panel with file tree
- Expand/collapse directories
- Create, rename, delete, and move files
- Auto-detection of project root
- Smart filtering (ignores node_modules, target, .git, etc.)

### Search & Replace
- In-file search with live preview
- Search next/previous
- Case-sensitive, whole-word, and regex modes
- Global search across entire project
- Global replace

### LSP Support (Language Server Protocol)
- Built-in LSP client for autocompletion
- Diagnostic display (errors, warnings)
- Keyword completion for supported languages
- Configurable LSP server per language
- Supported: Python (pylsp), JS/TS (typescript-language-server), Go (gopls), Rust (rust-analyzer), C/C++ (clangd)

### Git Integration
- View repository status
- See file diffs
- Create commits
- View current branch
- View commit log
- Show modified files

### Navigation
- Go to line
- Go to definition (LSP)
- Find references (LSP)
- Word navigation (forward/backward)
- Page up/down

## Installation

### From Source (Linux)

```bash
# Clone the repository
git clone https://github.com/ishmaelvertil/vertil-nano-pro.git
cd vertil-nano-pro

# Build release
chmod +x scripts/build.sh
./scripts/build.sh release

# Install system-wide
chmod +x scripts/install.sh
./scripts/install.sh
```

### From Source (Manual)

```bash
# Install Rust if not installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and install
cargo build --release
sudo cp target/release/nanopro /usr/local/bin/
```

### Termux (Android)

```bash
# Install Rust in Termux
pkg install rust cmake git

# Clone and build
git clone https://github.com/ishmaelvertil/vertil-nano-pro.git
cd vertil-nano-pro

# Use the Termux build script
chmod +x scripts/termux/build.sh
./scripts/termux/build.sh
```

### Uninstall

```bash
chmod +x scripts/uninstall.sh
./scripts/uninstall.sh
```

## Usage

```bash
# Open a file
nanopro myfile.py

# Open multiple files
nanopro file1.rs file2.go

# Create a new file
nanopro newfile.js

# Show about info
nanopro --about

# Show version
nanopro --version
```

## Keybindings

### File Operations
| Shortcut | Action |
|----------|--------|
| `Ctrl+O` | Save file |
| `Ctrl+S` | Save file (alternative) |
| `Ctrl+G` | Go to line |
| `Ctrl+Q` | Quit editor |
| `Ctrl+W` | Close tab / Search |

### Navigation
| Shortcut | Action |
|----------|--------|
| `↑ ↓ ← →` | Move cursor |
| `Shift+Arrows` | Select text |
| `Home / End` | Line start/end |
| `PageUp / PageDown` | Page scroll |
| `Alt+Left / Right` | Switch tabs |
| `Ctrl+L` | Go to line |

### Editing
| Shortcut | Action |
|----------|--------|
| `Enter` | New line (auto-indent) |
| `Backspace / Delete` | Delete character |
| `Tab / Shift+Tab` | Indent / Dedent |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+K` | Cut line/selection |
| `Ctrl+U` | Paste |
| `Ctrl+Shift+C` | Copy selection |
| `Ctrl+Shift+V` | Clipboard history |
| `Ctrl+A` | Select all |
| `Ctrl+D` | Duplicate line |
| `Ctrl+/` | Toggle comment |
| `Alt+Up / Down` | Move line up/down |

### Search & Replace
| Shortcut | Action |
|----------|--------|
| `Ctrl+W` | Search in file |
| `Ctrl+R` | Replace |
| `Ctrl+P` | Command palette |

### Project
| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | Toggle project explorer |
| `F1` | Help |

### Command Palette Commands
| Command | Description |
|---------|-------------|
| `:w` or `:save` | Save file |
| `:q` or `:quit` | Quit |
| `:wq` or `:x` | Save and quit |
| `:theme dark/light` | Change theme |
| `:git status` | Show git status |
| `:git diff [file]` | Show diff |
| `:git commit <msg>` | Create commit |
| `:git branch` | Show current branch |
| `:git log` | Show commit log |
| `:find <query>` | Search in project |
| `:ai` | AI Assistant (Coming Soon) |
| `:about` | Show about info |

## Configuration

Configuration is stored at `~/.config/nanopro/config.toml` (or `$XDG_CONFIG_HOME/nanopro/config.toml`).

Example configuration:

```toml
tab_size = 4
use_spaces = true
auto_indent = true
line_numbers = true
soft_wrap = false
auto_save = false
lsp_enabled = true
git_enabled = true
```

### Custom Themes

Place custom themes in `~/.config/nanopro/themes/`. See the included `themes/dark.toml` and `themes/light.toml` for reference.

## Performance

Vertil Nano Pro is built with performance as a top priority:

- **Startup time**: < 50ms on modern hardware
- **Memory usage**: Typically 10-30MB for normal files
- **No Electron dependency**: Pure native binary
- **No Node.js dependency**: Built with Rust
- **Efficient rendering**: Only redraws changed content
- **Line-based buffer**: Fast operations up to ~50k lines

## Architecture

Vertil Nano Pro is written in Rust and uses:
- **crossterm** — Cross-platform terminal control
- **tree-sitter** — Incremental syntax parsing
- **git2** — Git integration (libgit2 bindings)
- **tokio** — Async runtime for LSP communication
- **regex** — Fast regular expression search

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

## Roadmap

### V1 (Current)
- Modern functional editor
- Syntax highlighting with Tree-sitter
- LSP client for autocompletion and diagnostics
- Git integration
- Project explorer
- Modern selection and clipboard
- Search and replace (local and global)
- Themes

### V2 (Planned)
- Advanced refactoring tools
- Code navigation (go to definition, find references)
- Snippet expansion
- Multi-cursor editing
- Split view
- Plugin system

### V3 (Future)
- AI Assistant integration
- AI-powered code agents
- Code generation
- Smart suggestions
- Natural language editing

## License

MIT License — See [LICENSE](LICENSE) for details.

## Author

**Ishmael Vertil**

- Repository: https://github.com/ishmaelvertil/vertil-nano-pro

---

*Vertil Nano Pro — Simple. Fast. Professional.*
