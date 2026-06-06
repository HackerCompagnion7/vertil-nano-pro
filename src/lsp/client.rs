#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::editor::BufferPosition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    pub command: String,
    pub args: Vec<String>,
    pub file_patterns: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum LspNotification {
    Diagnostics(Vec<Diagnostic>),
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: (BufferPosition, BufferPosition),
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl From<u32> for DiagnosticSeverity {
    fn from(value: u32) -> Self {
        match value {
            1 => DiagnosticSeverity::Error,
            2 => DiagnosticSeverity::Warning,
            3 => DiagnosticSeverity::Information,
            4 => DiagnosticSeverity::Hint,
            _ => DiagnosticSeverity::Information,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompletionItemKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Keyword,
    Snippet,
    File,
    Folder,
    Enum,
    Constant,
    Struct,
    Unknown,
}

impl From<u32> for CompletionItemKind {
    fn from(value: u32) -> Self {
        match value {
            1 => CompletionItemKind::Text,
            2 => CompletionItemKind::Method,
            3 => CompletionItemKind::Function,
            4 => CompletionItemKind::Constructor,
            5 => CompletionItemKind::Field,
            6 => CompletionItemKind::Variable,
            7 => CompletionItemKind::Class,
            8 => CompletionItemKind::Interface,
            9 => CompletionItemKind::Module,
            10 => CompletionItemKind::Property,
            14 => CompletionItemKind::Keyword,
            15 => CompletionItemKind::Snippet,
            17 => CompletionItemKind::File,
            19 => CompletionItemKind::Folder,
            13 => CompletionItemKind::Enum,
            21 => CompletionItemKind::Constant,
            23 => CompletionItemKind::Struct,
            _ => CompletionItemKind::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub path: PathBuf,
    pub range: (BufferPosition, BufferPosition),
}

pub struct LspClient {
    server_configs: HashMap<String, LspConfig>,
    diagnostics: HashMap<PathBuf, Vec<Diagnostic>>,
    initialized: bool,
}

impl LspClient {
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        // Python - pylsp
        configs.insert(
            "python".to_string(),
            LspConfig {
                command: "pylsp".to_string(),
                args: vec![],
                file_patterns: vec!["*.py".to_string(), "*.pyw".to_string()],
            },
        );

        // JavaScript/TypeScript - typescript-language-server
        configs.insert(
            "javascript".to_string(),
            LspConfig {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                file_patterns: vec!["*.js".to_string(), "*.mjs".to_string(), "*.cjs".to_string()],
            },
        );

        configs.insert(
            "typescript".to_string(),
            LspConfig {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                file_patterns: vec!["*.ts".to_string(), "*.tsx".to_string()],
            },
        );

        // Go - gopls
        configs.insert(
            "go".to_string(),
            LspConfig {
                command: "gopls".to_string(),
                args: vec![],
                file_patterns: vec!["*.go".to_string()],
            },
        );

        // Rust - rust-analyzer
        configs.insert(
            "rust".to_string(),
            LspConfig {
                command: "rust-analyzer".to_string(),
                args: vec![],
                file_patterns: vec!["*.rs".to_string()],
            },
        );

        // C - clangd
        configs.insert(
            "c".to_string(),
            LspConfig {
                command: "clangd".to_string(),
                args: vec![],
                file_patterns: vec!["*.c".to_string(), "*.h".to_string()],
            },
        );

        // C++ - clangd
        configs.insert(
            "cpp".to_string(),
            LspConfig {
                command: "clangd".to_string(),
                args: vec![],
                file_patterns: vec!["*.cpp".to_string(), "*.cc".to_string(), "*.cxx".to_string(), "*.hpp".to_string()],
            },
        );

        Self {
            server_configs: configs,
            diagnostics: HashMap::new(),
            initialized: false,
        }
    }

    pub fn get_config(&self, language: &str) -> Option<&LspConfig> {
        self.server_configs.get(language)
    }

    pub fn is_language_supported(&self, language: &str) -> bool {
        self.server_configs.contains_key(language)
    }

    pub fn is_server_available(&self, language: &str) -> bool {
        if let Some(config) = self.server_configs.get(language) {
            // Check if the LSP server command exists
            which_exists(&config.command)
        } else {
            false
        }
    }

    pub fn update_diagnostics(&mut self, path: PathBuf, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.insert(path, diagnostics);
    }

    pub fn get_diagnostics(&self, path: &PathBuf) -> &[Diagnostic] {
        self.diagnostics.get(path).map_or(&[], |v| v)
    }

    pub fn clear_diagnostics(&mut self, path: &PathBuf) {
        self.diagnostics.remove(path);
    }

    pub fn get_completions(&self, _language: &str, _prefix: &str) -> Vec<CompletionItem> {
        // In V1, provide basic keyword completions without a running LSP server
        // Full LSP integration will use JSON-RPC over stdio pipes
        Vec::new()
    }

    pub fn get_keyword_completions(&self, language: &str, prefix: &str) -> Vec<CompletionItem> {
        let keywords = self.language_keywords(language);
        let lower_prefix = prefix.to_lowercase();

        keywords
            .iter()
            .filter(|kw| kw.to_lowercase().starts_with(&lower_prefix))
            .map(|kw| CompletionItem {
                label: kw.to_string(),
                kind: CompletionItemKind::Keyword,
                detail: None,
                documentation: None,
                insert_text: Some(kw.to_string()),
            })
            .collect()
    }

    fn language_keywords(&self, language: &str) -> Vec<&'static str> {
        match language {
            "python" => vec![
                "False", "None", "True", "and", "as", "assert", "async", "await",
                "break", "class", "continue", "def", "del", "elif", "else", "except",
                "finally", "for", "from", "global", "if", "import", "in", "is",
                "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
                "while", "with", "yield",
                "print", "range", "len", "str", "int", "float", "list", "dict",
                "set", "tuple", "bool", "type", "isinstance", "issubclass",
                "super", "property", "staticmethod", "classmethod",
            ],
            "javascript" | "typescript" => vec![
                "break", "case", "catch", "class", "const", "continue", "debugger",
                "default", "delete", "do", "else", "export", "extends", "false",
                "finally", "for", "function", "if", "import", "in", "instanceof",
                "new", "null", "return", "super", "switch", "this", "throw", "true",
                "try", "typeof", "undefined", "var", "void", "while", "with", "yield",
                "let", "async", "await", "of", "type", "interface", "enum",
                "implements", "private", "protected", "public", "abstract",
                "as", "from", "readonly", "static", "namespace", "declare",
                "console", "document", "window", "Promise", "Array", "Object",
                "String", "Number", "Boolean", "Map", "Set", "JSON",
            ],
            "go" => vec![
                "break", "case", "chan", "const", "continue", "default", "defer",
                "else", "fallthrough", "for", "func", "go", "goto", "if", "import",
                "interface", "map", "package", "range", "return", "select", "struct",
                "switch", "type", "var",
                "append", "cap", "close", "copy", "delete", "len", "make", "new",
                "panic", "print", "println", "recover",
                "fmt", "os", "io", "http", "strings", "strconv",
            ],
            "rust" => vec![
                "as", "async", "await", "break", "const", "continue", "crate", "dyn",
                "else", "enum", "extern", "fn", "for", "if", "impl", "in", "let",
                "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self",
                "Self", "static", "struct", "super", "trait", "type", "unsafe", "use",
                "where", "while", "yield",
                "Vec", "String", "Option", "Result", "Box", "Rc", "Arc",
                "println", "format", "vec", "panic", "unwrap", "expect",
                "HashMap", "HashSet", "BTreeMap", "BTreeSet",
            ],
            "c" => vec![
                "auto", "break", "case", "char", "const", "continue", "default", "do",
                "double", "else", "enum", "extern", "float", "for", "goto", "if",
                "int", "long", "register", "return", "short", "signed", "sizeof",
                "static", "struct", "switch", "typedef", "union", "unsigned", "void",
                "volatile", "while",
                "printf", "scanf", "malloc", "free", "calloc", "realloc",
                "NULL", "EOF", "stdin", "stdout", "stderr",
            ],
            "cpp" => vec![
                "alignas", "alignof", "and", "and_eq", "asm", "auto", "bitand",
                "bitor", "bool", "break", "case", "catch", "char", "char8_t",
                "char16_t", "char32_t", "class", "compl", "concept", "const",
                "consteval", "constexpr", "constinit", "const_cast", "continue",
                "co_await", "co_return", "co_yield", "decltype", "default", "delete",
                "do", "double", "dynamic_cast", "else", "enum", "explicit", "export",
                "extern", "false", "float", "for", "friend", "goto", "if", "inline",
                "int", "long", "mutable", "namespace", "new", "noexcept", "not",
                "not_eq", "nullptr", "operator", "or", "or_eq", "private", "protected",
                "public", "register", "reinterpret_cast", "requires", "return", "short",
                "signed", "sizeof", "static", "static_assert", "static_cast", "struct",
                "switch", "template", "this", "thread_local", "throw", "true", "try",
                "typedef", "typeid", "typename", "union", "unsigned", "using",
                "virtual", "void", "volatile", "wchar_t", "while", "xor", "xor_eq",
                "std", "cout", "cin", "endl", "vector", "string", "map", "set",
            ],
            _ => vec![],
        }
    }

    pub fn supported_languages(&self) -> Vec<&str> {
        self.server_configs.keys().map(|s| s.as_str()).collect()
    }
}

fn which_exists(command: &str) -> bool {
    std::process::Command::new("which")
        .arg(command)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

impl Default for LspClient {
    fn default() -> Self {
        Self::new()
    }
}
