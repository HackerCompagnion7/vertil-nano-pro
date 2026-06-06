use crossterm::style::Color;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub background: ColorDef,
    pub foreground: ColorDef,
    pub selection_bg: ColorDef,
    pub selection_fg: ColorDef,
    pub line_number_fg: ColorDef,
    pub line_number_active_fg: ColorDef,
    pub status_bar_bg: ColorDef,
    pub status_bar_fg: ColorDef,
    pub tab_bar_bg: ColorDef,
    pub tab_bar_fg: ColorDef,
    pub tab_active_bg: ColorDef,
    pub tab_active_fg: ColorDef,
    pub shortcut_bar_bg: ColorDef,
    pub shortcut_bar_fg: ColorDef,
    pub keyword: ColorDef,
    pub string_literal: ColorDef,
    pub comment: ColorDef,
    pub function_: ColorDef,
    pub type_: ColorDef,
    pub number: ColorDef,
    pub operator: ColorDef,
    pub variable: ColorDef,
    pub constant: ColorDef,
    pub property: ColorDef,
    pub punctuation: ColorDef,
    pub tag: ColorDef,
    pub attribute: ColorDef,
    pub escape_: ColorDef,
    pub error_: ColorDef,
    pub warning_: ColorDef,
    pub info_: ColorDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDef {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorDef {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_color(&self) -> Color {
        Color::Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "Vertil Dark".to_string(),
            background: ColorDef::rgb(30, 30, 46),
            foreground: ColorDef::rgb(205, 214, 244),
            selection_bg: ColorDef::rgb(68, 71, 110),
            selection_fg: ColorDef::rgb(205, 214, 244),
            line_number_fg: ColorDef::rgb(88, 91, 112),
            line_number_active_fg: ColorDef::rgb(166, 173, 200),
            status_bar_bg: ColorDef::rgb(49, 50, 68),
            status_bar_fg: ColorDef::rgb(205, 214, 244),
            tab_bar_bg: ColorDef::rgb(24, 24, 37),
            tab_bar_fg: ColorDef::rgb(166, 173, 200),
            tab_active_bg: ColorDef::rgb(30, 30, 46),
            tab_active_fg: ColorDef::rgb(137, 180, 250),
            shortcut_bar_bg: ColorDef::rgb(49, 50, 68),
            shortcut_bar_fg: ColorDef::rgb(166, 173, 200),
            keyword: ColorDef::rgb(203, 166, 247),       // Mauve
            string_literal: ColorDef::rgb(166, 227, 161), // Green
            comment: ColorDef::rgb(88, 91, 112),          // Surface2
            function_: ColorDef::rgb(137, 180, 250),      // Blue
            type_: ColorDef::rgb(249, 226, 175),          // Yellow
            number: ColorDef::rgb(250, 179, 135),         // Peach
            operator: ColorDef::rgb(137, 220, 235),       // Teal
            variable: ColorDef::rgb(205, 214, 244),       // Text
            constant: ColorDef::rgb(250, 179, 135),       // Peach
            property: ColorDef::rgb(166, 227, 161),       // Green
            punctuation: ColorDef::rgb(186, 194, 222),    // Subtext1
            tag: ColorDef::rgb(137, 180, 250),            // Blue
            attribute: ColorDef::rgb(249, 226, 175),      // Yellow
            escape_: ColorDef::rgb(245, 224, 220),        // Rosewater
            error_: ColorDef::rgb(243, 139, 168),         // Red
            warning_: ColorDef::rgb(249, 226, 175),       // Yellow
            info_: ColorDef::rgb(137, 180, 250),          // Blue
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Vertil Light".to_string(),
            background: ColorDef::rgb(239, 241, 245),
            foreground: ColorDef::rgb(76, 79, 105),
            selection_bg: ColorDef::rgb(220, 222, 232),
            selection_fg: ColorDef::rgb(76, 79, 105),
            line_number_fg: ColorDef::rgb(172, 176, 190),
            line_number_active_fg: ColorDef::rgb(76, 79, 105),
            status_bar_bg: ColorDef::rgb(204, 208, 218),
            status_bar_fg: ColorDef::rgb(76, 79, 105),
            tab_bar_bg: ColorDef::rgb(204, 208, 218),
            tab_bar_fg: ColorDef::rgb(92, 95, 119),
            tab_active_bg: ColorDef::rgb(239, 241, 245),
            tab_active_fg: ColorDef::rgb(30, 102, 245),
            shortcut_bar_bg: ColorDef::rgb(204, 208, 218),
            shortcut_bar_fg: ColorDef::rgb(92, 95, 119),
            keyword: ColorDef::rgb(136, 57, 239),       // Mauve
            string_literal: ColorDef::rgb(64, 160, 43),  // Green
            comment: ColorDef::rgb(172, 176, 190),       // Surface2
            function_: ColorDef::rgb(30, 102, 245),      // Blue
            type_: ColorDef::rgb(183, 135, 18),          // Yellow
            number: ColorDef::rgb(219, 85, 30),          // Peach
            operator: ColorDef::rgb(23, 146, 156),       // Teal
            variable: ColorDef::rgb(76, 79, 105),        // Text
            constant: ColorDef::rgb(219, 85, 30),        // Peach
            property: ColorDef::rgb(64, 160, 43),        // Green
            punctuation: ColorDef::rgb(108, 111, 133),   // Subtext1
            tag: ColorDef::rgb(30, 102, 245),            // Blue
            attribute: ColorDef::rgb(183, 135, 18),      // Yellow
            escape_: ColorDef::rgb(222, 100, 100),       // Red
            error_: ColorDef::rgb(210, 55, 75),          // Red
            warning_: ColorDef::rgb(183, 135, 18),       // Yellow
            info_: ColorDef::rgb(30, 102, 245),          // Blue
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: Theme,
    pub tab_size: usize,
    pub use_spaces: bool,
    pub auto_indent: bool,
    pub line_numbers: bool,
    pub soft_wrap: bool,
    pub word_wrap: bool,
    pub scroll_margin: usize,
    pub cursor_style: String,
    pub highlight_current_line: bool,
    pub show_whitespace: bool,
    pub auto_save: bool,
    pub auto_save_delay_ms: u64,
    pub lsp_enabled: bool,
    pub git_enabled: bool,
    pub explorer_width: usize,
    pub max_undo_history: usize,
    pub config_path: Option<PathBuf>,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            theme: Theme::dark(),
            tab_size: 4,
            use_spaces: true,
            auto_indent: true,
            line_numbers: true,
            soft_wrap: false,
            word_wrap: false,
            scroll_margin: 3,
            cursor_style: "block".to_string(),
            highlight_current_line: true,
            show_whitespace: false,
            auto_save: false,
            auto_save_delay_ms: 5000,
            lsp_enabled: true,
            git_enabled: true,
            explorer_width: 25,
            max_undo_history: 1000,
            config_path: None,
        }
    }

    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        let mut settings: Settings = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        settings.config_path = Some(path.clone());
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = self.config_path.as_ref().ok_or("No config path set")?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write config: {}", e))
    }

    pub fn load_or_create() -> Self {
        let config_dir = dirs_config_dir();
        if let Some(dir) = &config_dir {
            let config_path = dir.join("nanopro").join("config.toml");
            if config_path.exists() {
                if let Ok(settings) = Self::load(&config_path) {
                    return settings;
                }
            }
        }

        let mut settings = Self::new();
        if let Some(dir) = config_dir {
            let nanopro_dir = dir.join("nanopro");
            let _ = std::fs::create_dir_all(&nanopro_dir);
            settings.config_path = Some(nanopro_dir.join("config.toml"));
        }
        settings
    }

    pub fn set_theme(&mut self, theme_name: &str) {
        match theme_name {
            "light" | "Light" => self.theme = Theme::light(),
            _ => self.theme = Theme::dark(),
        }
    }
}

fn dirs_config_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg));
        }
    }
    std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config"))
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
