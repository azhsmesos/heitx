
pub struct FileType {
    name: String,
    hl_opts: HighlightingOptions,
}

#[derive(Default)]
pub struct HighlightingOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
    multiple_comments: bool,
    primary_keywords: Vec<String>,
    secondary_keywords: Vec<String>,
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("no filetype"),
            hl_opts: HighlightingOptions::default(),
        }
    }
}

impl FileType {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn from(filename: &str) -> Self {
        let rust_primary_keywords = vec![
            "as".to_string(),
            "break".to_string(),
            "const".to_string(),
            "continue".to_string(),
            "crate".to_string(),
            "else".to_string(),
            "enum".to_string(),
            "extern".to_string(),
            "false".to_string(),
            "fn".to_string(),
            "for".to_string(),
            "if".to_string(),
            "impl".to_string(),
            "in".to_string(),
            "let".to_string(),
            "loop".to_string(),
            "match".to_string(),
            "mod".to_string(),
            "move".to_string(),
            "mut".to_string(),
            "pub".to_string(),
            "ref".to_string(),
            "return".to_string(),
            "self".to_string(),
            "Self".to_string(),
            "static".to_string(),
            "struct".to_string(),
            "super".to_string(),
            "trait".to_string(),
            "true".to_string(),
            "type".to_string(),
            "unsafe".to_string(),
            "use".to_string(),
            "where".to_string(),
            "while".to_string(),
            "dyn".to_string(),
            "abstract".to_string(),
            "become".to_string(),
            "box".to_string(),
            "do".to_string(),
            "final".to_string(),
            "macro".to_string(),
            "override".to_string(),
            "priv".to_string(),
            "typeof".to_string(),
            "unsized".to_string(),
            "virtual".to_string(),
            "yield".to_string(),
            "async".to_string(),
            "await".to_string(),
            "try".to_string(),
        ];

        let rust_secondary_keywords = vec![
            "bool".to_string(),
            "char".to_string(),
            "i8".to_string(),
            "i16".to_string(),
            "i32".to_string(),
            "i64".to_string(),
            "isize".to_string(),
            "u8".to_string(),
            "u16".to_string(),
            "u32".to_string(),
            "u64".to_string(),
            "usize".to_string(),
            "f32".to_string(),
            "f64".to_string(),
        ];

        let java_primary_keywords = vec![
            "void".to_string(),
            "null".to_string(),
            "true".to_string(),
            "false".to_string(),
            "enum".to_string(),
            "public".to_string(),
            "protected".to_string(),
            "default".to_string(),
            "private".to_string(),
            "class".to_string(),
            "interface".to_string(),
            "abstract".to_string(),
            "implement".to_string(),
            "extends".to_string(),
            "new".to_string(),
            "import".to_string(),
            "package".to_string(),
            "if".to_string(),
            "else".to_string(),
            "while".to_string(),
            "for".to_string(),
            "switch".to_string(),
            "case".to_string(),
            "default".to_string(),
            "do".to_string(),
            "break".to_string(),
            "continue".to_string(),
            "return".to_string(),
            "instanceof".to_string(),
            "static".to_string(),
            "final".to_string(),
            "super".to_string(),
            "this".to_string(),
            "native".to_string(),
            "synchronized".to_string(),
            "volatile".to_string(),
            "const".to_string(),
        ];

        let java_secondary_keywords = vec![
            "byte".to_string(),
            "boolean".to_string(),
            "char".to_string(),
            "short".to_string(),
            "int".to_string(),
            "float".to_string(),
            "long".to_string(),
            "double".to_string(),
            "Object".to_string(),
        ];


        if filename.ends_with(".rs") {
            return Self {
                name: String::from("Rust"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    multiple_comments: true,
                    primary_keywords: rust_primary_keywords,
                    secondary_keywords: rust_secondary_keywords,
                },
            };
        }
        if filename.ends_with(".java") {
            return Self {
                name: String::from("Java"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    multiple_comments: true,
                    primary_keywords: java_primary_keywords,
                    secondary_keywords: java_secondary_keywords,
                }
            }
        }
        Self::default()
    }

    pub fn highlighting_options(&self) -> &HighlightingOptions {
        &self.hl_opts
    }
}

impl HighlightingOptions {
    pub fn numbers(&self) -> bool {
        self.numbers
    }

    pub fn strings(&self) -> bool {
        self.strings
    }

    pub fn characters(&self) -> bool {
        self.characters
    }

    pub fn comments(&self) -> bool {
        self.comments
    }

    pub fn primary_keywords(&self) -> &Vec<String> {
        &self.primary_keywords
    }

    pub fn secondary_keywords(&self) -> &Vec<String> {
        &self.secondary_keywords
    }

    pub fn multiple_comments(&self) -> bool {
        self.multiple_comments
    }
}