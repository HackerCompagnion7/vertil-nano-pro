# Vertil Nano Pro - Keybindings Reference

## Complete Keybinding Reference

### File Operations

| Shortcut | Action | Mode | Description |
|----------|--------|------|-------------|
| `Ctrl+O` | Save | Normal, Insert | Save current file |
| `Ctrl+S` | Save | Normal, Insert | Save current file (alternative) |
| `Ctrl+G` | Go to Line | Normal | Prompt for line number to jump to |
| `Ctrl+Q` | Quit | Normal | Exit the editor |
| `Ctrl+W` | Search / Close Tab | Normal | Open search (or close tab in some contexts) |

### Navigation

| Shortcut | Action | Mode | Description |
|----------|--------|------|-------------|
| `↑` (Up) | Move Up | All | Move cursor up one line |
| `↓` (Down) | Move Down | All | Move cursor down one line |
| `←` (Left) | Move Left | All | Move cursor left one character |
| `→` (Right) | Move Right | All | Move cursor right one character |
| `Home` | Line Start | All | Move to beginning of line |
| `End` | Line End | All | Move to end of line |
| `PageUp` | Page Up | All | Scroll up one page |
| `PageDown` | Page Down | All | Scroll down one page |
| `Alt+Left` | Previous Tab | Normal | Switch to previous tab |
| `Alt+Right` | Next Tab | Normal | Switch to next tab |
| `Ctrl+L` | Go to Line | Normal | Enter line number to jump to |

### Text Selection

| Shortcut | Action | Mode | Description |
|----------|--------|------|-------------|
| `Shift+↑` | Select Up | Normal | Extend selection upward |
| `Shift+↓` | Select Down | Normal | Extend selection downward |
| `Shift+←` | Select Left | Normal | Extend selection left |
| `Shift+→` | Select Right | Normal | Extend selection right |
| `Ctrl+A` | Select All | Normal | Select entire file contents |
| `Ctrl+Shift+L` | Select Line | Normal | Select current line |
| Mouse Drag | Select Range | Any | Click and drag to select text |

### Editing

| Shortcut | Action | Mode | Description |
|----------|--------|------|-------------|
| `Enter` | New Line | Insert | Insert new line with auto-indent |
| `Backspace` | Delete Back | Insert | Delete character before cursor |
| `Delete` | Delete Forward | Insert, Normal | Delete character after cursor |
| `Tab` | Indent | Insert | Insert tab or spaces |
| `Shift+Tab` | Dedent | Insert, Normal | Remove one level of indentation |
| `Ctrl+Z` | Undo | Normal, Insert | Undo last change |
| `Ctrl+Y` | Redo | Normal, Insert | Redo last undone change |
| `Ctrl+D` | Duplicate Line | Normal | Duplicate current line below |
| `Ctrl+/` | Toggle Comment | Normal | Comment/uncomment current line |
| `Alt+Up` | Move Line Up | Normal | Move current line up |
| `Alt+Down` | Move Line Down | Normal | Move current line down |

### Clipboard

| Shortcut | Action | Mode | Description |
|----------|--------|------|-------------|
| `Ctrl+K` | Cut | Normal | Cut selection or current line |
| `Ctrl+Shift+C` | Copy | Normal | Copy selection to clipboard |
| `Ctrl+U` | Paste | Normal | Paste from clipboard history |
| `Ctrl+Shift+V` | Clipboard History | Normal | Open clipboard history |

### Search & Replace

| Shortcut | Action | Mode | Description |
|----------|--------|------|-------------|
| `Ctrl+W` | Search | Normal | Open search prompt |
| `Ctrl+R` | Replace | Normal | Open replace prompt |
| `Ctrl+P` | Command Palette | Normal | Open command input |

### Project

| Shortcut | Action | Mode | Description |
|----------|--------|------|-------------|
| `Ctrl+T` | Toggle Explorer | Normal | Show/hide project file tree |
| `F1` | Help | Normal | Show help screen |

### Mode Switching

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Enter` | Normal → Insert | Enter insert mode from normal mode |
| `Esc` | Any → Normal | Return to normal mode from any mode |
| Typing a character | Normal → Insert | Start typing in insert mode |

### In Explorer Mode

| Shortcut | Action | Description |
|----------|--------|-------------|
| `↑ / ↓` | Navigate | Move selection up/down |
| `Enter` | Open / Expand | Open file or expand directory |
| `n` | New File | Create a new file |
| `d` | Delete | Delete selected file/directory |
| `Esc` | Close | Close explorer and return to editor |

### In Search Mode

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Enter` | Find Next | Jump to next match |
| `Esc` | Cancel | Cancel search and return |
| `Backspace` | Edit Pattern | Remove last character from search |
| Any character | Live Search | Add character and jump to match |

### In Go to Line Mode

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Enter` | Go | Jump to entered line number |
| `Esc` | Cancel | Cancel and return |
| `0-9` | Number Input | Enter line number |

## Command Palette Commands

Type `Ctrl+P` then enter a command:

| Command | Description |
|---------|-------------|
| `w` or `write` or `save` | Save current file |
| `q` or `quit` or `exit` | Quit the editor |
| `wq` or `x` | Save and quit |
| `theme dark` | Switch to dark theme (requires restart) |
| `theme light` | Switch to light theme (requires restart) |
| `git status` | Show git status |
| `git diff [file]` | Show diff for file (or all files) |
| `git commit <message>` | Stage all and commit |
| `git branch` | Show current branch |
| `git log` | Show recent commits |
| `find <query>` | Search across project |
| `ai` | AI Assistant (Coming Soon in V3) |
| `about` | Show editor information |
| `newfile <name>` | Create a new file in project |

## Mouse Operations

| Action | Operation |
|--------|-----------|
| Left Click | Position cursor |
| Left Drag | Select text |
| Right Click | Copy selection |
| Scroll Up | Scroll editor up |
| Scroll Down | Scroll editor down |

## Tips

1. **Quick Start**: Just type to enter insert mode. Press `Esc` to go back to normal mode.
2. **Save Often**: Use `Ctrl+O` or `Ctrl+S` at any time.
3. **Search Live**: As you type in search mode, the editor jumps to the first match.
4. **Copy Everything**: Use commands like `:copy line`, `:copy function`, `:copy class` in the command palette.
5. **Git Quick Commit**: `Ctrl+P` → `:git commit Your message here`
6. **Project Navigation**: `Ctrl+T` to open the file tree, navigate with arrows, `Enter` to open files.
