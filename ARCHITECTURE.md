# Vertil Nano Pro - Architecture Documentation

## Overview

Vertil Nano Pro is a terminal-based code editor built in Rust. The architecture prioritizes simplicity, performance, and maintainability while supporting modern features like LSP, syntax highlighting, and git integration.

## Design Principles

1. **Simplicity First** — Every module solves a real problem. No speculative complexity.
2. **Performance** — Fast startup, low memory, responsive input.
3. **No External Runtimes** — No Electron, no Node.js. Pure Rust binary.
4. **Modular Architecture** — Each module is independent and testable.
5. **Nano-like UX** — Easy to learn, Ctrl-based shortcuts, no modal editing.

## Module Architecture

```
src/
├── main.rs              # Entry point, CLI, event loop
├── editor/              # Core editor functionality
│   ├── buffer.rs        # Text buffer (storage, undo/redo, file I/O)
│   ├── cursor.rs        # Cursor position and movement
│   ├── selection.rs     # Text selection (character, line, block)
│   ├── tabs.rs          # Tab management (multiple buffers)
│   ├── view.rs          # Viewport/scroll management
│   └── mod.rs           # Module exports
├── ui/                  # Terminal rendering
│   ├── renderer.rs      # Crossterm-based rendering engine
│   └── mod.rs
├── syntax/              # Syntax highlighting
│   ├── highlight.rs     # Tree-sitter integration, highlight spans
│   └── mod.rs
├── lsp/                 # Language Server Protocol
│   ├── client.rs        # LSP client, completion, diagnostics
│   └── mod.rs
├── clipboard/           # Clipboard management
│   ├── manager.rs       # Clipboard history, system clipboard
│   └── mod.rs
├── git/                 # Git integration
│   ├── integration.rs   # Status, diff, commit via libgit2
│   └── mod.rs
├── search/              # Search and replace
│   ├── engine.rs        # Buffer search, global project search
│   └── mod.rs
├── project/             # Project explorer
│   ├── explorer.rs      # File tree, CRUD operations
│   └── mod.rs
└── config/              # Configuration
    ├── settings.rs      # Theme, preferences, persistence
    ├── keybindings.rs   # Keybinding definitions
    └── mod.rs
```

## Core Data Flow

```
User Input (Keyboard/Mouse)
        │
        ▼
   Event Loop (main.rs)
        │
        ▼
   Mode Handler
   ┌─────────────────────────────────┐
   │ Normal │ Insert │ Search │ ...  │
   └─────────────────────────────────┘
        │
        ▼
   Editor Operations
   ┌─────────────┐  ┌──────────┐  ┌───────────┐
   │   Buffer    │  │  Cursor  │  │ Selection │
   └─────────────┘  └──────────┘  └───────────┘
        │
        ▼
   Renderer (crossterm)
        │
        ▼
   Terminal Output
```

## Module Details

### Buffer (`editor/buffer.rs`)

The Buffer module is the core data structure. It stores text as a `Vec<String>` (one String per line), which provides:
- O(1) line access
- O(1) line count
- O(n) character count (cached when needed)
- Efficient insert/delete for normal editing patterns

**Key design decisions:**
- Line-based storage instead of rope — simpler, works well for files up to ~50k lines
- Separate undo/redo stacks with a maximum of 1000 entries
- File encoding detection (UTF-8 default)
- Language detection based on file extension

### Cursor (`editor/cursor.rs`)

Tracks cursor position (line, col) with a "preferred column" for vertical movement:
- When moving up/down, the cursor remembers its horizontal position
- When a line is shorter, the cursor clamps to line end but remembers the desired column
- Supports word movement (forward/backward) using whitespace boundaries

### Selection (`editor/selection.rs`)

Three selection modes:
1. **Character** — Standard text selection between two positions
2. **Line** — Full line selection (selects complete lines)
3. **Block** — Column/rectangular selection (for future use)

Selection maintains both an anchor point (where selection started) and a cursor point (current position). The selection persists during scrolling and auto-scrolls when the cursor moves beyond the viewport.

### View (`editor/view.rs`)

Manages the visible area of the buffer:
- Tracks scroll position (vertical and horizontal offsets)
- Ensures cursor visibility (auto-scroll)
- Calculates line number display width
- Supports soft wrap (configurable, off by default)
- Content area calculation (accounting for line numbers and margins)

### Renderer (`ui/renderer.rs`)

Renders the editor to the terminal using crossterm:
- Tab bar at the top
- Editor content with line numbers and syntax highlighting
- Status bar showing filename, language, cursor position
- Shortcut bar at the bottom
- Dialog rendering for modals
- Splash screen on startup
- Mouse-aware selection rendering

### Syntax Highlighting (`syntax/highlight.rs`)

Two-tier highlighting:
1. **Tree-sitter** — Primary method for supported languages. Parses the line and maps AST nodes to token types (keyword, string, comment, function, type, number, etc.)
2. **Fallback** — Simple regex-based highlighting for unsupported languages. Detects comments, strings, and common keywords.

Token types are mapped to colors via the active theme.

### LSP Client (`lsp/client.rs`)

Provides Language Server Protocol integration:
- Pre-configured LSP servers for Python, JS/TS, Go, Rust, C/C++
- Detects if LSP servers are available on the system
- Keyword completion as fallback when no LSP server is running
- Diagnostic collection and display
- Architecture prepared for full JSON-RPC over stdio communication (V2)

### Clipboard Manager (`clipboard/manager.rs`)

Modern clipboard with history:
- Stores up to 100 clipboard entries
- Each entry tracks its kind (text, line, block, function, class, file)
- System clipboard integration via xclip/wl-copy/pbcopy/termux-clipboard-set
- Paste by ID or index
- Preview formatting for clipboard history display

### Git Integration (`git/integration.rs`)

Git operations via libgit2 (git2 crate):
- Repository discovery and opening
- Status (new, modified, deleted, untracked files)
- Diff generation (additions, deletions, context)
- Commit creation (stage all + commit)
- Branch detection
- Commit log viewing

### Search Engine (`search/engine.rs`)

Two search systems:
1. **BufferSearch** — Search within the current file with case-sensitive, whole-word, and regex modes
2. **GlobalSearch** — Search across the entire project using walkdir for file traversal

Both support replacement with proper regex handling.

### Project Explorer (`project/explorer.rs`)

File tree navigation:
- Lazy loading of directory contents (expand on demand)
- Smart filtering (skips .git, node_modules, target, __pycache__, etc.)
- File operations: create, rename, delete, move
- Project root auto-detection (looks for .git, Cargo.toml, package.json, etc.)
- File search by name

### Configuration (`config/settings.rs`)

Settings system:
- TOML-based configuration file
- Theme definitions with RGB color values
- Catppuccin-based dark and light themes
- Auto-detection of config directory (XDG_CONFIG_HOME or ~/.config)
- Load-or-create pattern for first-run experience

## Performance Considerations

### Startup
- No heavy initialization at startup
- Tree-sitter parsers loaded on demand
- LSP connections established lazily
- Theme applied directly from compiled defaults

### Memory
- Line-based buffer: ~1 byte per character + overhead per line
- No background processes unless LSP is active
- Clipboard history capped at 100 entries
- Syntax highlighting computed per visible line only

### Rendering
- Only visible lines are rendered
- Horizontal scrolling avoids rendering off-screen content
- Batched terminal output via crossterm queue system
- Single flush per frame

## Future Architecture (V2/V3)

### V2: Advanced Navigation
- Full LSP JSON-RPC communication over stdio
- Go-to-definition and find-references
- Symbol search
- Multi-cursor support
- Split view architecture
- Plugin system with Lua scripting

### V3: AI Integration
- AI Assistant module
- Code generation pipeline
- Context management for large files
- Agent system for automated editing
- Natural language command processing

## Build System

### Release Build
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

These settings produce a minimal, optimized binary:
- LTO (Link-Time Optimization) removes unused code
- Single codegen unit enables maximum optimization
- Strip removes debug symbols
- Panic = abort reduces binary size by removing unwinding

### Cross-Compilation
The project supports cross-compilation for:
- `aarch64-linux-android` (Termux/Android)
- `x86_64-unknown-linux-gnu` (Linux)
- `x86_64-unknown-linux-musl` (Static Linux)
