use tree_sitter::{Parser, Query, QueryCursor, Tree};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub token_type: u32, // 1=keyword, 2=string, 3=comment, 4=function, 5=type, 6=number, 7=operator, 8=variable, 9=constant, 10=property, 11=punctuation, 12=tag, 13=attribute, 14=escape
}

pub struct HighlightEngine {
    parser: Mutex<Parser>,
}

impl HighlightEngine {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        // Default language will be set per-buffer
        Self {
            parser: Mutex::new(parser),
        }
    }

    pub fn highlight(&self, line: &str, language: &str) -> Vec<HighlightSpan> {
        if line.is_empty() {
            return Vec::new();
        }

        let language_id = match language {
            "python" => "python",
            "javascript" | "javascriptreact" => "javascript",
            "typescript" | "typescriptreact" => "typescript",
            "go" => "go",
            "rust" => "rust",
            "c" => "c",
            "cpp" => "cpp",
            _ => return self.fallback_highlight(line),
        };

        // Set the language for the parser
        let mut parser = self.parser.lock().unwrap();
        let language_obj = self.get_language(language_id);
        if language_obj.is_none() {
            return self.fallback_highlight(line);
        }

        if parser.set_language(language_obj.unwrap()).is_err() {
            return self.fallback_highlight(line);
        }

        let tree = parser.parse(line, None);
        match tree {
            Some(tree) => self.extract_spans(&tree, line),
            None => self.fallback_highlight(line),
        }
    }

    fn get_language(&self, lang_id: &str) -> Option<tree_sitter::Language> {
        match lang_id {
            "python" => Some(tree_sitter_python::language()),
            "javascript" => Some(tree_sitter_javascript::language()),
            "typescript" => Some(tree_sitter_typescript::language_typescript()),
            "go" => Some(tree_sitter_go::language()),
            "rust" => Some(tree_sitter_rust::language()),
            "c" => Some(tree_sitter_c::language()),
            "cpp" => Some(tree_sitter_cpp::language()),
            _ => None,
        }
    }

    fn extract_spans(&self, tree: &Tree, source: &str) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();
        let root = tree.root_node();

        self.walk_node(root, source, &mut spans);

        // Sort and merge overlapping spans
        spans.sort_by_key(|s| s.start);
        self.merge_overlapping(&mut spans);

        spans
    }

    fn walk_node(&self, node: tree_sitter::Node, source: &str, spans: &mut Vec<HighlightSpan>) {
        let token_type = self.node_to_token_type(&node);

        if token_type > 0 && node.child_count() == 0 {
            let start = node.start_byte();
            let end = node.end_byte();
            if start < end && end <= source.len() {
                spans.push(HighlightSpan {
                    start,
                    end,
                    token_type,
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_node(child, source, spans);
        }
    }

    fn node_to_token_type(&self, node: &tree_sitter::Node) -> u32 {
        let kind = node.kind();

        // Keywords
        match kind {
            // Python keywords
            "import" | "from" | "as" | "def" | "class" | "return" | "if" | "elif" | "else" |
            "for" | "while" | "try" | "except" | "finally" | "with" | "async" | "await" |
            "yield" | "lambda" | "pass" | "break" | "continue" | "raise" | "global" | "nonlocal" |
            "assert" | "del" | "in" | "not" | "and" | "or" | "is" | "True" | "False" | "None" |

            // JS/TS keywords
            "function" | "const" | "let" | "var" | "new" | "this" | "super" | "extends" |
            "typeof" | "instanceof" | "void" | "delete" | "throw" | "switch" | "case" |
            "default" | "do" | "export" | "import" | "from" | "as" | "type" | "interface" |
            "enum" | "implements" | "abstract" | "private" | "protected" | "public" |
            "static" | "readonly" | "namespace" | "module" | "declare" |

            // Go keywords
            "package" | "go" | "chan" | "select" | "range" | "defer" | "fallthrough" | "map" |

            // Rust keywords
            "fn" | "pub" | "use" | "mod" | "crate" | "self" | "Self" | "super" |
            "struct" | "enum" | "impl" | "trait" | "where" | "unsafe" | "extern" |
            "mut" | "ref" | "move" | "dyn" | "match" | "loop" | "macro_rules" |

            // C/C++ keywords
            "int" | "char" | "float" | "double" | "long" | "short" | "unsigned" | "signed" |
            "void" | "auto" | "register" | "volatile" | "extern" | "inline" | "sizeof" |
            "typedef" | "union" | "namespace" | "template" | "typename" | "virtual" |
            "override" | "final" | "constexpr" | "nullptr" | "static_cast" | "dynamic_cast" |
            "reinterpret_cast" | "const_cast" => 1, // keyword

            // Strings
            "string" | "string_literal" | "string_content" | "raw_string_literal" |
            "template_string" | "interpreted_string_literal" => 2,

            // Comments
            "comment" | "line_comment" | "block_comment" | "doc_comment" |
            "paragraph" => 3,

            // Functions/Methods
            "function_definition" | "function_declaration" | "method_definition" |
            "function_item" | "function_decl" | "method_declaration" | "constructor" |
            "decorator" | "call" | "call_expression" | "function_call" => 4,

            // Types
            "type_identifier" | "type" | "struct" | "enum" | "class_definition" |
            "class_declaration" | "interface_declaration" | "struct_item" | "enum_item" |
            "type_identifier" | "primitive_type" | "built_in_type" => 5,

            // Numbers
            "integer" | "float" | "number" | "integer_literal" | "float_literal" |
            "number_literal" | "hex_integer_literal" | "octal_integer_literal" |
            "binary_integer_literal" => 6,

            // Operators
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" |
            "&&" | "||" | "!" | "&" | "|" | "^" | "~" | "<<" | ">>" | "+=" | "-=" |
            "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | "++" | "--" |
            "->" | "=>" | "?" | "?." | "??=" | "||=" | "&&=" => 7,

            // Variables
            "identifier" | "variable_name" | "identifier_pattern" |
            "self_parameter" | "simple_identifier" => 8,

            // Constants
            "true" | "false" | "nil" | "null" | "undefined" | "YES" | "NO" => 9,

            // Properties
            "property_identifier" | "field_identifier" | "member_expression" |
            "attribute" | "property" | "field_expression" => 10,

            // Punctuation
            "(" | ")" | "{" | "}" | "[" | "]" | ";" | ":" | "," | "." | ".." | "..." => 11,

            // HTML/XML tags
            "tag_name" | "tag" | "start_tag" | "end_tag" | "self_closing_tag" => 12,

            // Attributes
            "attribute_name" | "attribute_value" | "decorator" => 13,

            // Escape sequences
            "escape_sequence" | "escape" => 14,

            _ => 0,
        }
    }

    fn merge_overlapping(&self, spans: &mut Vec<HighlightSpan>) {
        if spans.len() <= 1 {
            return;
        }

        let mut merged = Vec::new();
        merged.push(spans[0].clone());

        for span in spans.iter().skip(1) {
            let last = merged.last_mut().unwrap();
            if span.start <= last.end {
                // Overlapping: keep the one with higher priority (lower token_type = higher priority)
                if span.token_type > 0 && (last.token_type == 0 || span.start < last.start) {
                    last.end = last.end.max(span.end);
                    last.token_type = span.token_type;
                } else {
                    last.end = last.end.max(span.end);
                }
            } else {
                merged.push(span.clone());
            }
        }

        *spans = merged;
    }

    fn fallback_highlight(&self, line: &str) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();
        let trimmed = line.trim_start();

        // Simple comment detection
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("--") {
            spans.push(HighlightSpan {
                start: 0,
                end: line.len(),
                token_type: 3, // comment
            });
            return spans;
        }

        // Simple string detection
        let mut in_string = false;
        let mut string_start = 0;
        let mut string_char = '"';

        for (i, ch) in line.char_indices() {
            if !in_string && (ch == '"' || ch == '\'' || ch == '`') {
                in_string = true;
                string_start = i;
                string_char = ch;
            } else if in_string && ch == string_char {
                spans.push(HighlightSpan {
                    start: string_start,
                    end: i + 1,
                    token_type: 2, // string
                });
                in_string = false;
            }
        }

        // If unclosed string, highlight to end
        if in_string {
            spans.push(HighlightSpan {
                start: string_start,
                end: line.len(),
                token_type: 2,
            });
        }

        // Simple keyword detection for common languages
        let keywords = [
            "fn", "pub", "use", "mod", "struct", "enum", "impl", "trait", "let", "mut", "if",
            "else", "for", "while", "loop", "match", "return", "break", "continue", "where",
            "self", "Self", "super", "crate", "async", "await", "move", "ref", "type",
            "def", "class", "import", "from", "as", "with", "try", "except", "finally",
            "lambda", "yield", "pass", "raise", "global", "nonlocal", "assert", "del",
            "function", "const", "var", "new", "this", "typeof", "instanceof", "void",
            "switch", "case", "default", "do", "throw", "extends", "super",
            "package", "go", "chan", "select", "range", "defer",
            "int", "char", "float", "double", "long", "short", "unsigned", "signed",
            "inline", "sizeof", "typedef", "virtual", "override",
        ];

        let line_lower = line.to_lowercase();
        for keyword in &keywords {
            let search = format!(" {} ", keyword);
            if let Some(pos) = line_lower.find(&search) {
                let kw_start = pos + 1;
                let kw_end = kw_start + keyword.len();
                spans.push(HighlightSpan {
                    start: kw_start,
                    end: kw_end,
                    token_type: 1,
                });
            }
        }

        spans.sort_by_key(|s| s.start);
        spans
    }
}

impl Default for HighlightEngine {
    fn default() -> Self {
        Self::new()
    }
}
