use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    pub key: String,
    pub action: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingSet {
    pub bindings: HashMap<String, Keybinding>,
}

#[allow(dead_code)]
impl KeybindingSet {
    pub fn default_bindings() -> Self {
        let mut bindings = HashMap::new();

        // File operations
        bindings.insert("save".to_string(), Keybinding {
            key: "Ctrl+O".to_string(),
            action: "save".to_string(),
            description: "Save file".to_string(),
        });
        bindings.insert("save_as".to_string(), Keybinding {
            key: "Ctrl+Shift+O".to_string(),
            action: "save_as".to_string(),
            description: "Save as".to_string(),
        });
        bindings.insert("open".to_string(), Keybinding {
            key: "Ctrl+G".to_string(),
            action: "open".to_string(),
            description: "Open file".to_string(),
        });
        bindings.insert("quit".to_string(), Keybinding {
            key: "Ctrl+Q".to_string(),
            action: "quit".to_string(),
            description: "Quit editor".to_string(),
        });
        bindings.insert("close_tab".to_string(), Keybinding {
            key: "Ctrl+W".to_string(),
            action: "close_tab".to_string(),
            description: "Close tab".to_string(),
        });

        // Navigation
        bindings.insert("goto_line".to_string(), Keybinding {
            key: "Ctrl+L".to_string(),
            action: "goto_line".to_string(),
            description: "Go to line".to_string(),
        });
        bindings.insert("next_tab".to_string(), Keybinding {
            key: "Alt+Right".to_string(),
            action: "next_tab".to_string(),
            description: "Next tab".to_string(),
        });
        bindings.insert("prev_tab".to_string(), Keybinding {
            key: "Alt+Left".to_string(),
            action: "prev_tab".to_string(),
            description: "Previous tab".to_string(),
        });

        // Search
        bindings.insert("search".to_string(), Keybinding {
            key: "Ctrl+W".to_string(),
            action: "search".to_string(),
            description: "Search in file".to_string(),
        });
        bindings.insert("search_next".to_string(), Keybinding {
            key: "Alt+W".to_string(),
            action: "search_next".to_string(),
            description: "Search next".to_string(),
        });
        bindings.insert("search_prev".to_string(), Keybinding {
            key: "Alt+Shift+W".to_string(),
            action: "search_prev".to_string(),
            description: "Search previous".to_string(),
        });
        bindings.insert("global_search".to_string(), Keybinding {
            key: "Ctrl+Shift+F".to_string(),
            action: "global_search".to_string(),
            description: "Search in project".to_string(),
        });
        bindings.insert("replace".to_string(), Keybinding {
            key: "Ctrl+\\".to_string(),
            action: "replace".to_string(),
            description: "Replace".to_string(),
        });

        // Editing
        bindings.insert("undo".to_string(), Keybinding {
            key: "Ctrl+Z".to_string(),
            action: "undo".to_string(),
            description: "Undo".to_string(),
        });
        bindings.insert("redo".to_string(), Keybinding {
            key: "Ctrl+Y".to_string(),
            action: "redo".to_string(),
            description: "Redo".to_string(),
        });
        bindings.insert("cut".to_string(), Keybinding {
            key: "Ctrl+K".to_string(),
            action: "cut".to_string(),
            description: "Cut selection/line".to_string(),
        });
        bindings.insert("copy".to_string(), Keybinding {
            key: "Ctrl+Shift+C".to_string(),
            action: "copy".to_string(),
            description: "Copy selection".to_string(),
        });
        bindings.insert("paste".to_string(), Keybinding {
            key: "Ctrl+U".to_string(),
            action: "paste".to_string(),
            description: "Paste".to_string(),
        });
        bindings.insert("select_all".to_string(), Keybinding {
            key: "Ctrl+A".to_string(),
            action: "select_all".to_string(),
            description: "Select all".to_string(),
        });
        bindings.insert("select_line".to_string(), Keybinding {
            key: "Ctrl+Shift+L".to_string(),
            action: "select_line".to_string(),
            description: "Select line".to_string(),
        });
        bindings.insert("select_function".to_string(), Keybinding {
            key: "Ctrl+Shift+F".to_string(),
            action: "select_function".to_string(),
            description: "Select function".to_string(),
        });
        bindings.insert("comment_toggle".to_string(), Keybinding {
            key: "Ctrl+/".to_string(),
            action: "comment_toggle".to_string(),
            description: "Toggle comment".to_string(),
        });
        bindings.insert("duplicate_line".to_string(), Keybinding {
            key: "Ctrl+D".to_string(),
            action: "duplicate_line".to_string(),
            description: "Duplicate line".to_string(),
        });
        bindings.insert("move_line_up".to_string(), Keybinding {
            key: "Alt+Up".to_string(),
            action: "move_line_up".to_string(),
            description: "Move line up".to_string(),
        });
        bindings.insert("move_line_down".to_string(), Keybinding {
            key: "Alt+Down".to_string(),
            action: "move_line_down".to_string(),
            description: "Move line down".to_string(),
        });
        bindings.insert("indent".to_string(), Keybinding {
            key: "Tab".to_string(),
            action: "indent".to_string(),
            description: "Indent".to_string(),
        });
        bindings.insert("dedent".to_string(), Keybinding {
            key: "Shift+Tab".to_string(),
            action: "dedent".to_string(),
            description: "Dedent".to_string(),
        });

        // Project
        bindings.insert("toggle_explorer".to_string(), Keybinding {
            key: "Ctrl+T".to_string(),
            action: "toggle_explorer".to_string(),
            description: "Toggle project explorer".to_string(),
        });

        // Git
        bindings.insert("git_status".to_string(), Keybinding {
            key: "Ctrl+G".to_string(),
            action: "git_status".to_string(),
            description: "Git status".to_string(),
        });

        // Clipboard history
        bindings.insert("clipboard_history".to_string(), Keybinding {
            key: "Ctrl+Shift+V".to_string(),
            action: "clipboard_history".to_string(),
            description: "Clipboard history".to_string(),
        });

        // Command palette
        bindings.insert("command_palette".to_string(), Keybinding {
            key: "Ctrl+P".to_string(),
            action: "command_palette".to_string(),
            description: "Command palette".to_string(),
        });

        // Help
        bindings.insert("help".to_string(), Keybinding {
            key: "F1".to_string(),
            action: "help".to_string(),
            description: "Show help".to_string(),
        });

        Self { bindings }
    }

    pub fn get(&self, action: &str) -> Option<&Keybinding> {
        self.bindings.get(action)
    }

    pub fn all_bindings(&self) -> Vec<&Keybinding> {
        let mut bindings: Vec<&Keybinding> = self.bindings.values().collect();
        bindings.sort_by(|a, b| a.description.cmp(&b.description));
        bindings
    }
}

impl Default for KeybindingSet {
    fn default() -> Self {
        Self::default_bindings()
    }
}
